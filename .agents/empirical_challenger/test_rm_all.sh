#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)
CID_FILE="$TMP_WORKSPACE/cidfile"

# Run a dummy container to verify if it gets deleted
podman run -d --name dummy_test alpine sleep 100 > /dev/null

echo "--all" > "$CID_FILE"

# Trap execution
podman rm -f "$(cat "$CID_FILE")" >/dev/null 2>&1 || true

# Check if dummy container still exists
if podman ps -a | grep -q dummy_test; then
    echo "SAFE: --all was not evaluated as an option or didn't delete the container."
    podman rm -f dummy_test >/dev/null
else
    echo "VULNERABLE: podman rm -f --all deleted all containers!"
fi
rm -rf "$TMP_WORKSPACE"
