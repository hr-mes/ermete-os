#!/bin/bash
set -euo pipefail

QUALIFIED_KERNEL=""
for k in /lib/modules/*; do
    if [ -e "$k/vmlinuz" ] || [ -L "$k/vmlinuz" ]; then
        QUALIFIED_KERNEL=$(basename "$k")
        break
    fi
done

if [ -z "$QUALIFIED_KERNEL" ]; then
    echo "ERROR: No vmlinuz found in /lib/modules!"
    exit 1
fi

echo "Found Chimera Kernel: ${QUALIFIED_KERNEL}"

if [ -L "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz" ]; then
    REAL_VMLINUZ=$(readlink -f "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz")
    rm -f "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz"
    cp "$REAL_VMLINUZ" "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz"
fi

depmod "${QUALIFIED_KERNEL}"

echo "Generating Initramfs..."
dracut --no-hostonly --kver "${QUALIFIED_KERNEL}" --reproducible --compress "zstd -T0 -15" -v \
    --strip --omit-drivers "nouveau" \
    --add ostree --add fido2 --add tpm2-tss --add systemd-pcrphase \
    --install "/etc/group" --install "/etc/passwd" \
    -f "/usr/lib/modules/${QUALIFIED_KERNEL}/initramfs.img"

chmod 0644 "/usr/lib/modules/${QUALIFIED_KERNEL}/initramfs.img"

echo "Assembling Unified Kernel Image (UKI) using systemd-stub and ukify..."
mkdir -p /etc/pki/uki /boot/efi/EFI/Linux

KEY_SRC=""
if [ -f /run/secrets/uki_key ]; then
    KEY_SRC="/run/secrets/uki_key"
elif [ -f /run/secrets/uki-signing.key ]; then
    KEY_SRC="/run/secrets/uki-signing.key"
elif [ -f /run/secrets/uki.key ]; then
    KEY_SRC="/run/secrets/uki.key"
fi

if [ -z "$KEY_SRC" ]; then
    echo "ERROR: UKI signing key missing in /run/secrets!"
    exit 1
fi

CRT_SRC=""
if [ -f /run/secrets/uki_cert ]; then
    CRT_SRC="/run/secrets/uki_cert"
elif [ -f /run/secrets/uki_crt ]; then
    CRT_SRC="/run/secrets/uki_crt"
elif [ -f /run/secrets/uki-signing.crt ]; then
    CRT_SRC="/run/secrets/uki-signing.crt"
elif [ -f /run/secrets/uki.crt ]; then
    CRT_SRC="/run/secrets/uki.crt"
fi

if [ -z "$CRT_SRC" ]; then
    echo "ERROR: UKI signing certificate missing in /run/secrets!"
    exit 1
fi

echo "Installing UKI signing keys from /run/secrets/..."
cp "$KEY_SRC" /etc/pki/uki/uki-signing.key
chmod 0600 /etc/pki/uki/uki-signing.key
cp "$CRT_SRC" /etc/pki/uki/uki-signing.crt

STUB_PATH=$(find /usr/lib/systemd/boot/efi/ /usr/lib/systemd/ /usr/share/systemd/ -name "linuxx64.efi.stub" -o -name "systemd-stub.efi" 2>/dev/null | head -n 1)
if [ -z "$STUB_PATH" ]; then
    STUB_PATH="/usr/lib/systemd/boot/efi/linuxx64.efi.stub"
fi

UKIFY_BIN=$(command -v ukify || find /usr/lib/systemd /usr/bin -name "ukify" 2>/dev/null | head -n 1 || echo "ukify")
CMDLINE_STR="quiet splash fastboot iommu=pt intel_iommu=on amd_iommu=on efi=disable_early_pci_dma zswap.enabled=1 zswap.compressor=zstd rootflags=noatime slab_nomerge pti=on randomize_kstack_offset=on vsyscall=none debugfs=off oops=panic module.sig_enforce=1 lockdown=integrity init_on_free=1"

if command -v "$UKIFY_BIN" >/dev/null 2>&1 || [ -f "$UKIFY_BIN" ]; then
    "$UKIFY_BIN" build \
        --linux="/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz" \
        --initrd="/usr/lib/modules/${QUALIFIED_KERNEL}/initramfs.img" \
        --stub="$STUB_PATH" \
        --cmdline="$CMDLINE_STR" \
        --os-release="@/etc/os-release" \
        --secureboot-private-key=/etc/pki/uki/uki-signing.key \
        --secureboot-certificate=/etc/pki/uki/uki-signing.crt \
        --output="/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz.efi" || true
fi

if [ -f "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz.efi" ] && command -v sbsign >/dev/null 2>&1; then
    echo "Signing UKI EFI binary with sbsign..."
    sbsign --key /etc/pki/uki/uki-signing.key \
           --cert /etc/pki/uki/uki-signing.crt \
           --output "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz.efi.signed" \
           "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz.efi"
    mv -f "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz.efi.signed" "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz.efi"
fi

if [ -f "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz.efi" ]; then
    chmod 0755 "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz.efi"
    cp "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz.efi" /boot/efi/EFI/Linux/ermete-chimera-uki.efi
    cp "/usr/lib/modules/${QUALIFIED_KERNEL}/vmlinuz.efi" "/usr/lib/modules/${QUALIFIED_KERNEL}/uki.efi"
fi

ldconfig
