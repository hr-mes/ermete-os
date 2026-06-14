#!/bin/bash
set -euo pipefail
if [ ! -d "/var/home" ]; then exit 0; fi
mkdir -p /var/home/.snapshots
chmod 700 /var/home/.snapshots
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
btrfs subvolume snapshot /var/home "/var/home/.snapshots/home_$TIMESTAMP" || exit 0
cd /var/home/.snapshots
ls -1d home_* 2>/dev/null | sort -r | tail -n +4 | xargs -r btrfs subvolume delete || true
