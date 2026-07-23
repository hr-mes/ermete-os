#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)
echo "Workspace: $TMP_WORKSPACE"
# Simulate subuid folder with chattr or chmod 000 containing a file
podman unshare bash -c "mkdir -p $TMP_WORKSPACE/evil && touch $TMP_WORKSPACE/evil/file && chmod 000 $TMP_WORKSPACE/evil"

# Now host tries to delete it normally
rm -rf "$TMP_WORKSPACE" 2>/dev/null
if [ -d "$TMP_WORKSPACE/evil" ]; then
    echo "Normal rm failed (expected)."
else
    echo "Normal rm succeeded?"
fi

# Now host tries with podman unshare
podman unshare rm -rf "$TMP_WORKSPACE"
if [ -d "$TMP_WORKSPACE/evil" ]; then
    echo "podman unshare rm -rf FAILED!"
else
    echo "podman unshare rm -rf SUCCEEDED."
fi
