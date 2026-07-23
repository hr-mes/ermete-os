#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)
CID_FILE="$TMP_WORKSPACE/cidfile"

# Run a container with --rm and --cidfile
podman run --rm --cidfile="$CID_FILE" registry.fedoraproject.org/fedora:latest echo "Hello" >/dev/null 2>&1

# Check if cidfile exists
if [ -f "$CID_FILE" ]; then
    echo "CID_FILE exists!"
    cat "$CID_FILE"
else
    echo "CID_FILE was deleted by podman!"
fi
rm -rf "$TMP_WORKSPACE"
