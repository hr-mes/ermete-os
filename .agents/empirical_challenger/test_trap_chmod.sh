#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)
# We need to simulate files created by podman in a namespace.
# Let's create a file and directory, and make them unreadable.
mkdir -p "$TMP_WORKSPACE/dir"
touch "$TMP_WORKSPACE/dir/file"
chmod 000 "$TMP_WORKSPACE/dir"

# User space rm -rf will fail because of chmod 000
rm -rf "$TMP_WORKSPACE" 2>/dev/null
if [ -d "$TMP_WORKSPACE" ]; then
    echo "Regular rm -rf failed (expected)"
    
    # Now try podman unshare rm -rf
    podman unshare rm -rf "$TMP_WORKSPACE"
    if [ ! -d "$TMP_WORKSPACE" ]; then
        echo "PASSED: podman unshare rm -rf succeeded"
    else
        echo "FAILED: podman unshare rm -rf failed to delete the directory"
    fi
else
    echo "Regular rm -rf succeeded? Unexpected."
fi
