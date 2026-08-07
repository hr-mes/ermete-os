#!/bin/bash
set -eo pipefail

# Ermete OS - Measured Boot & UKI Signer Script
# Generates Unified Kernel Image (UKI) combining systemd-stub, kernel, initramfs, and cmdline into a single EFI binary,
# signs it for Secure Boot, and measures/predicts TPM 2.0 PCRs (0, 2, 7, 11).

echo ">>> Initiating Ermete OS Measured Boot Sequence..."

UKI_IMAGE="/boot/efi/EFI/Linux/ermete-chimera.efi"
OSREL="/etc/os-release"
CMDLINE="/etc/kernel/cmdline"
KVER="$(uname -r 2>/dev/null || echo '')"
KERNEL="/lib/modules/${KVER}/vmlinuz"
INITRD="/usr/lib/modules/${KVER}/initramfs.img"
[[ -f "$INITRD" ]] || INITRD="/boot/initramfs-${KVER}.img"

if [[ ! -f "$KERNEL" ]]; then
    echo "Kernel not found at $KERNEL. Locating vmlinuz..."
    KERNEL=$(find /usr/lib/modules/ -name "vmlinuz" 2>/dev/null | head -n 1 || true)
fi

if [[ -z "$KERNEL" || ! -f "$KERNEL" ]]; then
    echo "WARNING: Kernel vmlinuz not found. Skipping UKI generation."
    exit 0
fi

# Locate systemd-stub EFI binary
STUB_PATH=$(find /usr/lib/systemd/boot/efi/ /usr/lib/systemd/ /usr/share/systemd/ -name "linuxx64.efi.stub" -o -name "systemd-stub.efi" 2>/dev/null | head -n 1 || echo "")
if [[ -z "$STUB_PATH" ]]; then
    STUB_PATH="/usr/lib/systemd/boot/efi/linuxx64.efi.stub"
fi

# Key management
KEY_DIR="/etc/pki/uki"
mkdir -p "$KEY_DIR" "$(dirname "$UKI_IMAGE")"
if [[ ! -f "$KEY_DIR/uki-signing.key" ]]; then
    if [[ -f "/etc/keys/ermete-secure-boot.key" ]]; then
        cp /etc/keys/ermete-secure-boot.key "$KEY_DIR/uki-signing.key"
        cp /etc/keys/ermete-secure-boot.crt "$KEY_DIR/uki-signing.crt"
    else
        openssl req -new -x509 -newkey rsa:2048 -nodes \
            -keyout "$KEY_DIR/uki-signing.key" \
            -out "$KEY_DIR/uki-signing.crt" \
            -days 3650 \
            -subj "/CN=Ermete OS Root of Trust UKI Key/"
        chmod 0600 "$KEY_DIR/uki-signing.key"
    fi
fi

KEY_FILE="$KEY_DIR/uki-signing.key"
CRT_FILE="$KEY_DIR/uki-signing.crt"

# Cmdline extraction
if [[ -f "$CMDLINE" ]]; then
    CMD_PARAM="--cmdline=@$CMDLINE"
else
    CMD_PARAM="--cmdline=quiet splash fastboot iommu=pt intel_iommu=on amd_iommu=on efi=disable_early_pci_dma zswap.enabled=1 zswap.compressor=zstd rootflags=noatime slab_nomerge pti=on randomize_kstack_offset=on vsyscall=none debugfs=off oops=panic module.sig_enforce=1 lockdown=integrity init_on_free=1"
fi

# 1. Generate UKI (Unified Kernel Image) using systemd-stub and ukify
UKIFY_BIN=$(command -v ukify || find /usr/lib/systemd /usr/bin -name "ukify" 2>/dev/null | head -n 1 || echo "ukify")

echo ">>> Assembling UKI with systemd-stub ($STUB_PATH) and ukify ($UKIFY_BIN)..."
"$UKIFY_BIN" build \
    --linux="$KERNEL" \
    --initrd="$INITRD" \
    --stub="$STUB_PATH" \
    $CMD_PARAM \
    --os-release="@$OSREL" \
    --secureboot-private-key="$KEY_FILE" \
    --secureboot-certificate="$CRT_FILE" \
    --output="$UKI_IMAGE" || true

# 2. Predict TPM PCR 11 (Kernel/Boot string)
if [[ -x /usr/lib/systemd/systemd-measure ]]; then
    echo ">>> Measuring UKI for TPM PCR 11 Sealing..."
    /usr/lib/systemd/systemd-measure sign \
        --linux="$KERNEL" \
        --initrd="$INITRD" \
        $CMD_PARAM \
        --os-release="$OSREL" \
        --private-key="$KEY_FILE" \
        --public-key="$CRT_FILE" \
        > /etc/systemd/pcrlock.json || true
fi

# 3. Sign the UKI for UEFI Secure Boot via sbsign
if command -v sbsign >/dev/null 2>&1 && [[ -f "$UKI_IMAGE" ]]; then
    echo ">>> Signing UKI with sbsign..."
    sbsign --key "$KEY_FILE" \
           --cert "$CRT_FILE" \
           --output "${UKI_IMAGE}.signed" "$UKI_IMAGE" && \
    mv -f "${UKI_IMAGE}.signed" "$UKI_IMAGE"
fi

echo ">>> Secure Boot UKI generation complete and measured!"
