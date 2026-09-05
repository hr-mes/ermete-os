#!/usr/bin/env bash
# Gate del kernel guest (docs/architecture/doc_kernel_build.md, sezione 9 e fase K6): il
# vmlinux del pacchetto ermete-kernel-microvm avvia in Firecracker con una rootfs di prova
# (ext4 con busybox e microvm/init) e chiude con `K6 RESULT ok` sulla seriale, dopo le
# asserzioni di init (uname, root su virtio-blk, BTF, file system, dm-verity, niente
# moduli, kCFI, dmesg pulito). Gira nell'immagine boot/Containerfile e vuole /dev/kvm:
# Firecracker non ha un modo emulato.
#
# Uso: microvm/boot.sh --rpms DIR --out DIR
#   --rpms  directory in cui cercare ermete-kernel-microvm-*.rpm (l'out di build.sh)
#   --out   log della seriale e riepilogo
set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
RPMS='' OUT=''
while [[ $# -gt 0 ]]; do
  case $1 in
    --rpms) RPMS=$2; shift 2 ;;
    --out) OUT=$2; shift 2 ;;
    *) echo "argomento sconosciuto: $1" >&2; exit 2 ;;
  esac
done
[[ $RPMS && $OUT ]] || { echo "uso: microvm/boot.sh --rpms DIR --out DIR" >&2; exit 2; }
[[ -w /dev/kvm ]] || { echo "/dev/kvm non accessibile: Firecracker ha solo KVM" >&2; exit 2; }

die() { echo "errore: $*" >&2; exit 1; }
step() { echo; echo "== $*"; }

mapfile -t RPM < <(find "$RPMS" -name 'ermete-kernel-microvm-*.rpm')
[[ ${#RPM[@]} -eq 1 ]] || die "atteso un solo ermete-kernel-microvm-*.rpm in $RPMS, trovati ${#RPM[@]}"
WORK=$(mktemp -d)
mkdir -p "$OUT"

step "vmlinux da ${RPM[0]##*/}"
(cd "$WORK" && rpm2cpio "${RPM[0]}" | cpio -idm --quiet)
DIR=$WORK/usr/lib/ermete/microvm
[[ -s $DIR/vmlinux && -s $DIR/release ]] || die "vmlinux o release assenti nel pacchetto"
RELEASE=$(< "$DIR/release")
echo "kernel $RELEASE, vmlinux $(du -h "$DIR/vmlinux" | cut -f1), bzImage $(du -h "$DIR/bzImage" | cut -f1)"

step "rootfs di prova (ext4, busybox, microvm/init)"
R=$WORK/rootfs
mkdir -p "$R"/{bin,dev,proc,sys,tmp}
install -m 755 /usr/sbin/busybox "$R/bin/busybox"
for applet in $(/usr/sbin/busybox --list); do ln -s busybox "$R/bin/$applet"; done
install -m 755 "$HERE/init" "$R/init"
truncate -s 64M "$WORK/rootfs.ext4"
mkfs.ext4 -q -F -d "$R" "$WORK/rootfs.ext4"

step "Firecracker ($(firecracker --version | head -n 1))"
# Riga di comando come la documenta Firecracker: seriale come console, reboot via
# i8042 (e' cosi' che il guest fa uscire Firecracker), niente PCI, root su virtio-blk.
cat > "$WORK/vm.json" <<JSON
{
  "boot-source": {
    "kernel_image_path": "$DIR/vmlinux",
    "boot_args": "console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw init=/init k6.release=$RELEASE"
  },
  "drives": [
    { "drive_id": "rootfs", "path_on_host": "$WORK/rootfs.ext4", "is_root_device": true, "is_read_only": false }
  ],
  "machine-config": { "vcpu_count": 2, "mem_size_mib": 512 }
}
JSON
LOG=$OUT/firecracker.log
timeout 180 firecracker --no-api --config-file "$WORK/vm.json" > "$LOG" 2>&1 || echo "firecracker: uscita $?"

{
  echo "## MicroVM $RELEASE in Firecracker"; echo; echo "| caso | esito |"; echo "| --- | --- |"
  if grep -q '^K6 RESULT ok' "$LOG"; then echo "| firecracker | ok |"; else echo "| firecracker | FAIL |"; fi
} > "$OUT/summary.md"
step "riepilogo"; cat "$OUT/summary.md"
grep -q '^K6 RESULT ok' "$LOG" || { grep -E '^K6 (FAIL|RESULT)' "$LOG" || tail -n 30 "$LOG"; die "il kernel guest non ha chiuso con K6 RESULT ok"; }
