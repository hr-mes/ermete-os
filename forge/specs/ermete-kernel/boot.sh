#!/usr/bin/env bash
# Boot matrix del kernel Ermete (docs/architecture/doc_kernel_build.md, sezione 7, gate 3).
# Gira nell'immagine boot/Containerfile. Dal kernel-core RPM estrae vmlinuz, costruisce un
# initramfs di prova (busybox, bpftool, boot/init) e una UKI firmata con una MOK effimera,
# arruola la MOK nel varstore OVMF con Secure Boot acceso e avvia QEMU quattro volte:
# firmware {SeaBIOS, OVMF+Secure Boot via shim} x CPU {Nehalem, host}. Nehalem prova che
# nel kernel non e' entrata nessuna istruzione oltre x86-64 baseline. Ogni avvio deve
# terminare con `K3 RESULT ok` sulla seriale (le asserzioni sono in boot/init).
#
# Uso: boot.sh --rpms DIR --out DIR [--accel kvm|tcg] [--case NOME]...
#   --rpms   directory in cui cercare kernel-core-*.rpm (l'out di build.sh o l'artefatto)
#   --out    log seriali, riepilogo e materiale di prova
#   --accel  kvm (default, serve /dev/kvm) o tcg (emulazione: lento, `host` diventa `max`)
#   --case   limita la matrice (ripetibile): bios-nehalem bios-host uefi-nehalem uefi-host
set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
RPMS='' OUT='' ACCEL=kvm CASES=()
while [[ $# -gt 0 ]]; do
  case $1 in
    --rpms) RPMS=$2; shift 2 ;;
    --out) OUT=$2; shift 2 ;;
    --accel) ACCEL=$2; shift 2 ;;
    --case) CASES+=("$2"); shift 2 ;;
    *) echo "argomento sconosciuto: $1" >&2; exit 2 ;;
  esac
done
[[ $RPMS && $OUT ]] || { echo "uso: boot.sh --rpms DIR --out DIR [--accel kvm|tcg] [--case NOME]..." >&2; exit 2; }
[[ ${#CASES[@]} -gt 0 ]] || CASES=(bios-nehalem bios-host uefi-nehalem uefi-host)
[[ $ACCEL == kvm && ! -w /dev/kvm ]] && { echo "/dev/kvm non accessibile: usa --accel tcg" >&2; exit 2; }

die() { echo "errore: $*" >&2; exit 1; }
step() { echo; echo "== $*"; }

mapfile -t CORE < <(find "$RPMS" -name 'kernel-core-*.rpm')
[[ ${#CORE[@]} -eq 1 ]] || die "atteso un solo kernel-core-*.rpm in $RPMS, trovati ${#CORE[@]}"
KVER=$(rpm -qp --qf '%{VERSION}-%{RELEASE}.%{ARCH}' "${CORE[0]}")
CMDLINE=$(< "$HERE/cmdline")
# Solo per la prova: console seriale, riavvio immediato su panic (con -no-reboot QEMU
# esce), una policy IMA che misuri qualcosa, e i parametri letti da boot/init.
TEST_CMDLINE="$CMDLINE console=ttyS0,115200 panic=-1 ima_policy=tcb k3.uname=$KVER"

WORK=$(mktemp -d)
mkdir -p "$OUT"
step "kernel $KVER da ${CORE[0]##*/}"
mkdir -p "$WORK/rpm" && (cd "$WORK/rpm" && rpm2cpio "${CORE[0]}" | cpio -idm --quiet "./lib/modules/$KVER/vmlinuz")
VMLINUZ="$WORK/rpm/lib/modules/$KVER/vmlinuz"
[[ -s $VMLINUZ ]] || die "vmlinuz assente nel kernel-core"

step "initramfs di prova"
R="$WORK/initramfs"
mkdir -p "$R"/{bin,dev,proc,sys,tmp,usr/sbin}
install -m 755 /usr/sbin/busybox "$R/bin/busybox"
# Link relativi: `busybox --install` li farebbe assoluti verso $R, che nel guest non esiste.
for applet in $(/usr/sbin/busybox --list); do ln -s busybox "$R/bin/$applet"; done
# bpftool con le sue librerie: quello di Fedora si porta dietro libLLVM (140 MB in
# chiaro, 39 MB compressi), il prezzo di `bpftool feature probe` fatto con lo strumento vero.
install -m 755 /usr/sbin/bpftool "$R/usr/sbin/bpftool"
# Tutte in /lib64, il percorso di default del loader: nel guest non c'e' ld.so.cache e
# libLLVM sta in una directory che sull'host entra solo tramite ld.so.conf.d.
ldd /usr/sbin/bpftool | awk '/=> \//{print $3} /^\s*\/lib64\/ld-linux/{print $1}' \
  | while read -r lib; do install -D "$lib" "$R/lib64/${lib##*/}"; done
install -m 755 "$HERE/boot/init" "$R/init"
(cd "$R" && find . | cpio -o -H newc --quiet | zstd -q -T0 -19 -o "$WORK/initramfs.img")
echo "initramfs: $(du -sh "$R" | cut -f1) in chiaro, $(du -h "$WORK/initramfs.img" | cut -f1) compresso"

step "UKI firmata con una MOK effimera, arruolata nel varstore OVMF"
openssl req -x509 -newkey rsa:2048 -nodes -days 2 -subj '/CN=Ermete OS K3 test MOK/' \
  -keyout "$WORK/mok.key" -out "$OUT/mok.pem" 2> /dev/null
ukify build --linux "$VMLINUZ" --initrd "$WORK/initramfs.img" --uname "$KVER" \
  --cmdline "$TEST_CMDLINE k3.sb=1" --stub /usr/lib/systemd/boot/efi/linuxx64.efi.stub \
  --signtool sbsign --secureboot-private-key "$WORK/mok.key" --secureboot-certificate "$OUT/mok.pem" \
  --output "$WORK/uki.efi" > "$OUT/ukify.log"
sbverify --cert "$OUT/mok.pem" "$WORK/uki.efi" >> "$OUT/ukify.log"
OVMF_CODE=/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd
virt-fw-vars -i /usr/share/edk2/ovmf/OVMF_VARS.secboot.fd -o "$WORK/vars.fd" \
  --add-mok "$(< /proc/sys/kernel/random/uuid)" "$OUT/mok.pem" > "$OUT/varstore.log"
# ESP: shim al percorso removibile, la UKI dove shim cerca il secondo stadio.
mkdir -p "$WORK/esp/EFI/BOOT"
cp /boot/efi/EFI/fedora/shimx64.efi "$WORK/esp/EFI/BOOT/BOOTX64.EFI"
cp "$WORK/uki.efi" "$WORK/esp/EFI/BOOT/grubx64.efi"

run_case() { # run_case NOME  (NOME = <bios|uefi>-<nehalem|host>)
  local name=$1 fw=${1%-*} cpu=${1#*-} log="$OUT/$1.log" args
  [[ $cpu == host && $ACCEL == tcg ]] && cpu=max
  [[ $cpu == nehalem ]] && cpu=Nehalem
  args=(-machine "q35,smm=on" -accel "$ACCEL" -cpu "$cpu" -smp 2 -m 1024
        -display none -monitor none -serial "file:$log" -no-reboot
        -device virtio-rng-pci)
  case $fw in
    bios) args+=(-kernel "$VMLINUZ" -initrd "$WORK/initramfs.img" -append "$TEST_CMDLINE") ;;
    uefi) cp "$WORK/vars.fd" "$WORK/vars-$name.fd"
          args+=(-global "driver=cfi.pflash01,property=secure,value=on"
                 -drive "if=pflash,format=raw,readonly=on,file=$OVMF_CODE"
                 -drive "if=pflash,format=raw,file=$WORK/vars-$name.fd"
                 -drive "if=virtio,format=raw,readonly=on,file=fat:ro:$WORK/esp") ;;
    *) die "caso sconosciuto: $name" ;;
  esac
  step "$name (firmware $fw, cpu $cpu, accel $ACCEL)"
  timeout 900 qemu-system-x86_64 "${args[@]}" || echo "qemu: uscita $?"
  if grep -q '^K3 RESULT ok' "$log"; then
    RESULTS+=("| $name | ok |"); echo "$name: ok"
  else
    RESULTS+=("| $name | FAIL |"); FAILED+=("$name")
    echo "$name: FAIL"; grep -E '^K3 (FAIL|RESULT)' "$log" || tail -n 20 "$log"
  fi
}

RESULTS=() FAILED=()
for c in "${CASES[@]}"; do run_case "$c"; done

{
  echo "## Boot matrix $KVER (accel $ACCEL)"; echo; echo "| caso | esito |"; echo "| --- | --- |"
  printf '%s\n' "${RESULTS[@]}"
} > "$OUT/summary.md"
step "riepilogo"; cat "$OUT/summary.md"
[[ ${#FAILED[@]} -eq 0 ]] || die "casi falliti: ${FAILED[*]}"
