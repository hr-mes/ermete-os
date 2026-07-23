#!/bin/bash
set -euo pipefail

TIMESTAMP=$(date +%Y%m%d-%H%M%S)

echo "Triggering Btrfs Time-Warp snapshot for user data..."

if mountpoint -q /var/home; then
    mkdir -p /var/home/.snapshots
    btrfs subvolume snapshot -r /var/home "/var/home/.snapshots/home-$TIMESTAMP" || true
fi

if mountpoint -q /var/lib; then
    mkdir -p /var/lib/.snapshots
    btrfs subvolume snapshot -r /var/lib "/var/lib/.snapshots/lib-$TIMESTAMP" || true
fi

echo "Time-Warp snapshots created successfully."
