#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# 🌋 Ermete Forge - Deterministic Hermetic Build Script (Nix-Paradigm)
# ==============================================================================
# Forces builds into a hermetic environment without network access using bwrap.
# All downloaded dependencies must be pre-fetched and verified against a lockfile (sha256).

LOCKFILE="${1:-ermete-build.lock}"
WORKSPACE_DIR="$(pwd)"

echo "=> 🌋 Ermete Hermetic Build System (Nix-Paradigm)"

if [ ! -f "$LOCKFILE" ]; then
    echo "ERROR: Lockfile '$LOCKFILE' not found."
    echo "Deterministic builds require explicit dependency locking and pre-fetching."
    exit 1
fi

echo "=> Verifying dependencies against $LOCKFILE..."
# Assuming lockfile contains standard sha256sum output
if ! sha256sum --check "$LOCKFILE" --quiet; then
    echo "ERROR: Checksum mismatch! Dependencies have been altered or are incomplete."
    exit 1
fi
echo "=> Dependencies verified successfully."

echo "=> Entering hermetic sandbox (bwrap, no-net)..."

# Use bubblewrap to create a completely isolated environment without network.
# Only the workspace is mounted read-write.
bwrap \
    --unshare-all \
    --ro-bind /usr /usr \
    --dir /tmp \
    --dir /var \
    --proc /proc \
    --dev /dev \
    --symlink usr/lib /lib \
    --symlink usr/lib64 /lib64 \
    --symlink usr/bin /bin \
    --symlink usr/sbin /sbin \
    --bind "$WORKSPACE_DIR" /workspace \
    --chdir /workspace \
    /bin/bash -c "
        echo 'Inside hermetic sandbox.'
        ip link || echo 'Network is successfully isolated.'
        echo 'Executing build...'
        if [ -f "./build.sh" ]; then
            ./build.sh
        else
            echo "Executing hermetic build script..."
            ./build.sh
        fi
    "

echo "=> Hermetic build completed successfully."
