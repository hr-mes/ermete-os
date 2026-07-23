#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)

# Try to create a hardlink to /etc/shadow inside the workspace
# Note: /etc/shadow is on the host. The container only sees its own /etc/shadow or nothing if not mounted.
# If they create a hardlink to the CONTAINER'S /etc/shadow, we don't care, it's just the container's shadow.
# Can they create a hardlink to the HOST'S /etc/shadow? They don't have it mounted!
# Wait, they don't have the host filesystem mounted at all!
echo "Testing hardlink from within container..."
podman run --rm -v "$TMP_WORKSPACE:$TMP_WORKSPACE:Z" registry.fedoraproject.org/fedora:latest sh -c "ln /etc/passwd $TMP_WORKSPACE/artifact.json" || true

if [ -f "$TMP_WORKSPACE/artifact.json" ]; then
    echo "Hardlink created!"
    cat "$TMP_WORKSPACE/artifact.json" | head -n 1
else
    echo "Hardlink failed."
fi

rm -rf "$TMP_WORKSPACE"
