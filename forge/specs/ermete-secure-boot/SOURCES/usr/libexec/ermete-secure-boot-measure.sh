#!/bin/bash
set -eo pipefail

# Ermete OS - Measured Boot & UKI Signer Script
# This script is meant to be run inside the `ermete-os-builder` during the kernel build phase
# or via a systemd service if signing updates locally.

echo ">>> Initiating Ermete OS Measured Boot Sequence..."

UKI_IMAGE="/boot/efi/EFI/Linux/ermete-chimera.efi"
OSREL="/etc/os-release"
CMDLINE="/etc/kernel/cmdline"
KERNEL="/lib/modules/$(uname -r)/vmlinuz"
INITRD="/boot/initramfs-$(uname -r).img"

if [[ ! -f "$KERNEL" ]]; then
    echo "Kernel not found. Skipping."
    exit 0
fi

# 1. Generate UKI (Unified Kernel Image) using systemd-ukify
echo ">>> Building UKI with ukify..."
/usr/lib/systemd/ukify build \
    --linux="$KERNEL" \
    --initrd="$INITRD" \
    --cmdline="@$CMDLINE" \
    --os-release="@$OSREL" \
    --output="$UKI_IMAGE"

# 2. Predict TPM PCR 11 (Kernel/Boot string)
# systemd-measure pre-calculates what the TPM PCRs will look like when this UKI boots.
echo ">>> Measuring UKI for TPM PCR 11 Sealing..."
/usr/lib/systemd/systemd-measure sign \
    --linux="$KERNEL" \
    --initrd="$INITRD" \
    --cmdline="$CMDLINE" \
    --os-release="$OSREL" \
    --private-key=/etc/keys/ermete-secure-boot.key \
    --public-key=/etc/keys/ermete-secure-boot.crt \
    > /etc/systemd/pcrlock.json

# 3. Sign the UKI for UEFI Secure Boot
# Requires sbsigntools.
echo ">>> Signing UKI with sbsign..."
sbsign --key /etc/keys/ermete-secure-boot.key \
       --cert /etc/keys/ermete-secure-boot.crt \
       --output "$UKI_IMAGE" "$UKI_IMAGE"

echo ">>> Secure Boot UKI generation complete and measured!"
