#!/bin/bash
set -euo pipefail

# ==============================================================================
# Ermete OS - TPM 2.0 PCR Sealing for LUKS /var/home
# Seals LUKS partition to TPM2 PCRs 0 (firmware), 2 (option ROMs), 7 (Secure Boot state), 11 (UKI/Kernel measurement)
# ==============================================================================

echo ">>> Initiating Ermete OS TPM 2.0 PCR Sealing for LUKS /var/home (PCRs 0,2,7,11)..."

# Check systemd-cryptenroll availability
if ! command -v systemd-cryptenroll >/dev/null 2>&1; then
    echo "WARNING: systemd-cryptenroll is not installed. Skipping TPM2 LUKS sealing."
    exit 0
fi

# Detect LUKS target partition for /var/home or default block devices
TARGET_DEV=""

# Option 1: Find partition mounted on /var/home or /home or backing LUKS device
if [ -d "/var/home" ]; then
    MOUNT_DEV=$(df --output=source /var/home 2>/dev/null | tail -n 1 || true)
    if [[ "$MOUNT_DEV" =~ ^/dev/mapper/ ]]; then
        MAP_NAME=$(basename "$MOUNT_DEV")
        SLAVE_DEV=$(ls /sys/block/dm-*/slaves 2>/dev/null | grep "$MAP_NAME" || true)
        if [ -n "$SLAVE_DEV" ]; then
            PHYS_NAME=$(ls "/sys/block/$(echo "$SLAVE_DEV" | cut -d/ -f4)/slaves" 2>/dev/null | head -n 1 || true)
            if [ -n "$PHYS_NAME" ]; then
                TARGET_DEV="/dev/$PHYS_NAME"
            fi
        fi
    elif [[ "$MOUNT_DEV" =~ ^/dev/ ]]; then
        TARGET_DEV="$MOUNT_DEV"
    fi
fi

# Option 2: Search blkid for crypto_LUKS partitions
if [ -z "$TARGET_DEV" ] || [ ! -b "$TARGET_DEV" ]; then
    TARGET_DEV=$(blkid -t TYPE=crypto_LUKS -o device 2>/dev/null | grep -E '/dev/vda|/dev/nvme|/dev/sda' | head -n 1 || true)
fi

# Option 3: Fallback check for /dev/vda3, /dev/vda2, etc.
if [ -z "$TARGET_DEV" ] || [ ! -b "$TARGET_DEV" ]; then
    for candidate in /dev/vda3 /dev/vda2 /dev/vda1 /dev/nvme0n1p3 /dev/sda3; do
        if [ -b "$candidate" ]; then
            TARGET_DEV="$candidate"
            break
        fi
    done
fi

if [ -z "$TARGET_DEV" ] || [ ! -b "$TARGET_DEV" ]; then
    echo "NOTICE: No active block device found for /var/home LUKS partition. Defaulting to /dev/vda3."
    TARGET_DEV="/dev/vda3"
fi

echo "Selected LUKS device for TPM2 PCR sealing: $TARGET_DEV"

# Execute systemd-cryptenroll with TPM2 PCRs 0, 2, 7, 11
if [ -b "$TARGET_DEV" ]; then
    echo "Running systemd-cryptenroll --tpm2-device=auto --tpm2-pcrs=0,2,7,11 $TARGET_DEV ..."
    systemd-cryptenroll --tpm2-device=auto --tpm2-pcrs=0,2,7,11 "$TARGET_DEV" || {
        echo "WARNING: systemd-cryptenroll failed or TPM2 hardware device is absent in this execution environment."
        exit 0
    }
    echo ">>> Successfully sealed LUKS partition $TARGET_DEV to TPM 2.0 PCRs 0,2,7,11!"
else
    echo "WARNING: Block device $TARGET_DEV does not exist. Skipping runtime execution."
fi

exit 0
