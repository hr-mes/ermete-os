#!/usr/bin/env bash
set -euo pipefail

# Script: consolidate_fuzzing.sh
# Removes duplicate fuzzing directories from forge/specs and consolidates into tests/fuzz/

FUZZ_DIRS=(
    "forge/specs/ermete-backup/ermete-backup-1.0.0/fuzz"
    "forge/specs/ermete-cloud-rs/ermete-cloud-rs-1.0.0/fuzz"
    "forge/specs/ermete-daemon-rs/ermete-daemon-rs-0.2.1/fuzz"
    "forge/specs/ermete-gatekeeper-rs/ermete-gatekeeper-rs-1.0.0/fuzz"
    "forge/specs/ermete-lvfs-rs/ermete-lvfs-rs-1.0.0/fuzz"
    "forge/specs/ermete-mdm-rs/ermete-mdm-rs-1.0.0/fuzz"
    "forge/specs/ermete-settings-rs/ermete-settings-rs-1.0.0/fuzz"
    "forge/specs/ermete-shell-rs/ermete-shell-rs-1.0.0/fuzz"
    "forge/specs/ermete-store-rs/ermete-store-rs-1.0.0/fuzz"
)

echo "=== Eliminazione directory fuzzing ridondanti in forge/specs ==="
for dir in "${FUZZ_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        echo "Rimuovendo: $dir"
        rm -rf "$dir"
    else
        echo "Già rimossa/non trovata: $dir"
    fi
done

echo "=== Consolidamento completato in tests/fuzz/ ==="
