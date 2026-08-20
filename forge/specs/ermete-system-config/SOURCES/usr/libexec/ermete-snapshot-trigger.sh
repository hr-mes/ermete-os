#!/bin/bash
set -euo pipefail

# Ermete OS: Zero-Trust Time-Warp Engine
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
RETENTION_LIMIT=15

echo "[Time-Warp] Inizializzazione snapshot atomica Bcachefs..."

create_snapshot() {
    local TARGET_DIR=$1
    local PREFIX=$2
    local SNAP_DIR="${TARGET_DIR}/.snapshots"

    if mountpoint -q "$TARGET_DIR"; then
        # 1. Zero-Trust: Verifica che il filesystem sia effettivamente Bcachefs
        FS_TYPE=$(df -T "$TARGET_DIR" | tail -n 1 | awk '{print $2}')
        if [ "$FS_TYPE" != "bcachefs" ]; then
            echo "[Time-Warp] CRITICO: Il filesystem su $TARGET_DIR non è bcachefs (rilevato: $FS_TYPE). Snapshot atomica impossibile."
            exit 1
        fi

        mkdir -p "$SNAP_DIR"
        local NEW_SNAP="${SNAP_DIR}/${PREFIX}-${TIMESTAMP}"
        
        echo "[Time-Warp] Creazione subvolume read-only: $NEW_SNAP"
        # Rimozione dell'ignobile "|| true" (Silent Failure)
        bcachefs subvolume snapshot -r "$TARGET_DIR" "$NEW_SNAP"

        # 2. Rotazione degli snapshot (Retention Policy)
        # Conta quanti snapshot esistono per questo prefisso
        local COUNT=$(ls -1d "${SNAP_DIR}/${PREFIX}-"* 2>/dev/null | wc -l || echo 0)
        
        if [ "$COUNT" -gt "$RETENTION_LIMIT" ]; then
            local TO_DELETE=$((COUNT - RETENTION_LIMIT))
            echo "[Time-Warp] Rotazione attivata: eliminazione dei $TO_DELETE snapshot più vecchi..."
            
            ls -1d "${SNAP_DIR}/${PREFIX}-"* | head -n "$TO_DELETE" | while read -r old_snap; do
                echo "[Time-Warp] Eliminazione subvolume: $old_snap"
                bcachefs subvolume delete "$old_snap"
            done
        fi
    else
        echo "[Time-Warp] $TARGET_DIR non è un mountpoint, skip."
    fi
}

create_snapshot "/var/home" "home"
create_snapshot "/var/lib" "lib"

echo "[Time-Warp] 🟢 Operazione Time-Warp conclusa con successo."
