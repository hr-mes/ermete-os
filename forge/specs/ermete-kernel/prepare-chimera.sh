#!/bin/bash
set -euo pipefail
echo ">>> [ZERO-TRUST] Preparazione Kernel Offline"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
if [ -d "$SCRIPT_DIR/cachyos-patches" ]; then
    mkdir -p /tmp/cachyos-patches/
    \cp -R -f "$SCRIPT_DIR/cachyos-patches"/* /tmp/cachyos-patches/ || true
fi
echo ">>> [ZERO-TRUST] Kernel pronto per RPMBUILD."
