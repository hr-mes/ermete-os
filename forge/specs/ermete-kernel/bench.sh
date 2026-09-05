#!/usr/bin/env bash
# Benchmark di tendenza del kernel (docs/architecture/doc_kernel_build.md, sezione 7,
# gate 5; fase K7). Avvia il kernel-core in QEMU/KVM con un initramfs di prova che porta
# hackbench, schbench, fio e netperf (bench/init li esegue in sequenza e stampa una riga
# `K7 <metrica> <valore> <unita'>` per ciascuno), poi raccoglie le righe in
# OUT/results.json e OUT/summary.md. Gira nell'immagine boot/Containerfile e vuole
# /dev/kvm: con TCG i numeri misurerebbero l'emulatore. Non e' un gate: e' il numero che
# decide -O3 e ogni futura opzione, confrontato tra settimane (bench-report.py) o tra due
# kernel nello stesso run (--label).
#
# Uso: bench.sh --rpms DIR --out DIR [--label NOME] [--seconds N] [--cpus N] [--mem MiB]
#               [--accel kvm|tcg]
#   --rpms     directory con kernel-core-*.rpm
#   --out      results.json, summary.md, log seriale
#   --label    nome del kernel nel risultato (default: il kver)
#   --seconds  durata di ogni prova (default 30: cinque prove, meno di cinque minuti)
#   --accel    kvm (default) o tcg: solo per provare l'initramfs, i numeri non valgono
set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
RPMS='' OUT='' LABEL='' SECONDS_PER_TEST=30 CPUS=4 MEM=4096 ACCEL=kvm
while [[ $# -gt 0 ]]; do
  case $1 in
    --rpms) RPMS=$2; shift 2 ;;
    --out) OUT=$2; shift 2 ;;
    --label) LABEL=$2; shift 2 ;;
    --seconds) SECONDS_PER_TEST=$2; shift 2 ;;
    --cpus) CPUS=$2; shift 2 ;;
    --mem) MEM=$2; shift 2 ;;
    --accel) ACCEL=$2; shift 2 ;;
    *) echo "argomento sconosciuto: $1" >&2; exit 2 ;;
  esac
done
[[ $RPMS && $OUT ]] || { echo "uso: bench.sh --rpms DIR --out DIR [--label NOME] [--seconds N] [--cpus N] [--mem MiB] [--accel kvm|tcg]" >&2; exit 2; }
[[ $ACCEL == kvm && ! -w /dev/kvm ]] && { echo "/dev/kvm non accessibile: senza KVM il benchmark misura l'emulatore (--accel tcg solo per provare)" >&2; exit 2; }
CPU=host; [[ $ACCEL == tcg ]] && CPU=max

die() { echo "errore: $*" >&2; exit 1; }
step() { echo; echo "== $*"; }

mapfile -t CORE < <(find "$RPMS" -name 'kernel-core-*.rpm')
[[ ${#CORE[@]} -eq 1 ]] || die "atteso un solo kernel-core-*.rpm in $RPMS, trovati ${#CORE[@]}"
KVER=$(rpm -qp --qf '%{VERSION}-%{RELEASE}.%{ARCH}' "${CORE[0]}")
LABEL=${LABEL:-$KVER}
WORK=$(mktemp -d)
mkdir -p "$OUT"

step "kernel $KVER da ${CORE[0]##*/}"
mkdir -p "$WORK/rpm" && (cd "$WORK/rpm" && rpm2cpio "${CORE[0]}" | cpio -idm --quiet "./lib/modules/$KVER/vmlinuz")
VMLINUZ="$WORK/rpm/lib/modules/$KVER/vmlinuz"
[[ -s $VMLINUZ ]] || die "vmlinuz assente nel kernel-core"

step "initramfs con gli strumenti"
R="$WORK/initramfs"
mkdir -p "$R"/{bin,dev,proc,sys,tmp,usr/bin}
install -m 755 /usr/sbin/busybox "$R/bin/busybox"
for applet in $(/usr/sbin/busybox --list); do ln -s busybox "$R/bin/$applet"; done
# Gli eseguibili con le loro librerie in /lib64 (nel guest non c'e' ld.so.cache).
for tool in hackbench schbench fio netperf netserver; do
  path=$(command -v "$tool") || die "$tool non trovato nell'immagine"
  install -m 755 "$path" "$R/usr/bin/$tool"
  ldd "$path" | awk '/=> \//{print $3} /^\s*\/lib64\/ld-linux/{print $1}' \
    | while read -r lib; do [[ -e "$R/lib64/${lib##*/}" ]] || install -D "$lib" "$R/lib64/${lib##*/}"; done
done
install -m 755 "$HERE/bench/init" "$R/init"
(cd "$R" && find . | cpio -o -H newc --quiet | zstd -q -T0 -19 -o "$WORK/initramfs.img")

step "QEMU ($ACCEL): $CPUS vCPU, $MEM MiB, $SECONDS_PER_TEST s per prova"
LOG="$OUT/serial.log"
timeout $(( SECONDS_PER_TEST * 8 + 300 )) qemu-system-x86_64 -machine q35 -accel "$ACCEL" -cpu "$CPU" -smp "$CPUS" -m "$MEM" \
  -display none -monitor none -serial "file:$LOG" -no-reboot -device virtio-rng-pci \
  -kernel "$VMLINUZ" -initrd "$WORK/initramfs.img" \
  -append "console=ttyS0,115200 panic=-1 quiet k7.seconds=$SECONDS_PER_TEST" || echo "qemu: uscita $?"
grep -q '^K7 RESULT ok' "$LOG" || { grep -E '^K7 ' "$LOG" || tail -n 30 "$LOG"; die "il benchmark non ha chiuso con K7 RESULT ok"; }

step "risultati"
host_cpu=$(awk -F': ' '/^model name/ { print $2; exit }' /proc/cpuinfo)
python3 - "$LOG" "$OUT/results.json" "$LABEL" "$KVER" "$host_cpu" "$CPUS" "$MEM" "$SECONDS_PER_TEST" <<'PY'
import json, re, sys, datetime
log, out, label, kver, cpu, cpus, mem, seconds = sys.argv[1:]
metrics, failed = {}, []
for line in open(log, errors="replace"):
    m = re.match(r"^K7 (\S+) (.+) (\S+)$", line.strip())
    if not m or m.group(1) in ("START", "RESULT"):
        continue
    try:
        value = float(m.group(2))
    except ValueError:
        value = float("nan")
    if value != value:  # nan: la prova non ha prodotto un numero (il testo e' nel log seriale)
        failed.append(m.group(1))
    else:
        metrics[m.group(1)] = {"value": value, "unit": m.group(3)}
result = {"label": label, "kver": kver, "date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d"),
          "host_cpu": cpu, "vcpus": int(cpus), "mem_mib": int(mem), "seconds_per_test": int(seconds), "metrics": metrics}
json.dump(result, open(out, "w"), indent=2)
lines = [f"## Benchmark {label} ({kver})", "", f"QEMU/KVM {cpus} vCPU, {mem} MiB, {seconds} s per prova, host `{cpu}`", "",
         "| metrica | valore | unita' |", "| --- | --- | --- |"]
lines += [f"| {k} | {v['value']:g} | {v['unit']} |" for k, v in metrics.items()]
lines += [f"| {k} | n/d | |" for k in failed]
open(out.replace("results.json", "summary.md"), "w").write("\n".join(lines) + "\n")
print("\n".join(lines))
if failed:
    sys.exit("prove senza numero: " + ", ".join(failed))
PY
