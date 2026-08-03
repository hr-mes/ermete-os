#!/usr/bin/env bash
set -euo pipefail

NV_INDEX="0x01800001"
BUILD_FILE="/etc/os-release"

if ! command -v tpm2_nvincrement &>/dev/null; then
    exit 0
fi

CURRENT_BUILD=$(grep -E "^BUILD_ID=" "$BUILD_FILE" 2>/dev/null | cut -d= -f2 | tr -d '"' || true)
CURRENT_BUILD=${CURRENT_BUILD:-1}

COUNTER_RAW=$(tpm2_nvread "$NV_INDEX" -C o 2>/dev/null | xxd -p | tr -d '\n' || echo "")
if [ -n "$COUNTER_RAW" ]; then
    COUNTER_VAL=$((16#$COUNTER_RAW))
    if [ "$CURRENT_BUILD" -gt "$COUNTER_VAL" ]; then
        echo "Incremento del TPM2 Monotonic Counter a $CURRENT_BUILD..."
        tpm2_nvincrement "$NV_INDEX" -C o
    fi
fi
