#!/bin/bash
mkdir -p /var/home/ermete/GEMINI/ermete/.agents/empirical_challenger/cids
CID_FILE="/var/home/ermete/GEMINI/ermete/.agents/empirical_challenger/cids/test_cid"
TMP_WORKSPACE=$(mktemp -d)

# Start a background container that simulates a short-running task
podman run --rm --cidfile="$CID_FILE" -v "$TMP_WORKSPACE:$TMP_WORKSPACE:Z" alpine sleep 2 >/dev/null 2>&1 &
PODMAN_PID=$!

# Simulate daemon crash
echo "Daemon crashed!"

# Wait for container to finish naturally
wait $PODMAN_PID

# Simulate daemon restart (Startup recovery scan)
echo "Daemon restarting..."
if [ -f "$CID_FILE" ]; then
    echo "Found cidfile, cleaning up..."
    # clean up logic
else
    echo "No cidfile found!"
fi

if [ -d "$TMP_WORKSPACE" ]; then
    echo "VULNERABILITY FOUND: Workspace $TMP_WORKSPACE leaked!"
    rm -rf "$TMP_WORKSPACE"
    exit 1
else
    echo "Workspace was cleaned up."
fi
