#!/usr/bin/env bash
set -euo pipefail

NV_INDEX="0x01800001"
BUILD_FILE="/etc/os-release"

if ! command -v tpm2_nvread &>/dev/null; then
    echo "WARNING: tpm2-tools non presenti. Skipping TPM rollback check."
    exit 0
fi

# Estrazione versione/build corrente dal SO
if [ -f "$BUILD_FILE" ]; then
    CURRENT_BUILD=$(grep -E "^BUILD_ID=" "$BUILD_FILE" | cut -d= -f2 | tr -d '"' || true)
fi
CURRENT_BUILD=${CURRENT_BUILD:-1}

# Lettura valore counter monotonico dal TPM2
COUNTER_RAW=$(tpm2_nvread "$NV_INDEX" -C o 2>/dev/null | xxd -p | tr -d '\n' || echo "")

if [ -z "$COUNTER_RAW" ]; then
    echo "NOTICE: NV Index TPM2 $NV_INDEX non ancora definito."
    exit 0
fi

COUNTER_VAL=$((16#$COUNTER_RAW))

echo "TPM Rollback Check: Version Current=$CURRENT_BUILD, TPM Counter=$COUNTER_VAL"

if [ "$CURRENT_BUILD" -lt "$COUNTER_VAL" ]; then
    echo "CRITICAL: Rilevato Attacco di Rollback! La versione del sistema ($CURRENT_BUILD) e' inferiore al Counter TPM ($COUNTER_VAL)."
    echo "Spegnimento immediato del sistema per protezione dati."
    systemctl poweroff -ff
    exit 1
fi

echo "TPM Rollback Check OK."
exit 0
