#!/bin/bash

# Setup mock environment
TEST_DIR="/tmp/ermete_test_env"
mkdir -p "$TEST_DIR/integration/state/workspaces"

# Mock the daemon creating a workspace
TMP_WORKSPACE=$(mktemp -d -p "$TEST_DIR/integration/state/workspaces/")

# Mock the agent running in the container
# The agent creates a world-writable artifact
echo '{"status": "success"}' > "$TMP_WORKSPACE/forge_status.json"
chmod 777 "$TMP_WORKSPACE/forge_status.json"
# We can't easily mock subuid ownership here without root/podman unshare,
# but the permission leak (0777) is enough to prove the vulnerability.

# Mock the daemon's validation
WORKSPACE_REAL=$(realpath "$TMP_WORKSPACE")
artifact_path="$TMP_WORKSPACE/forge_status.json"
resolved_path=$(realpath "$artifact_path")

if [[ "$resolved_path" == "$WORKSPACE_REAL/"* ]]; then
    if [ -f "$artifact_path" ]; then
        # Daemon uses mktemp to create a secure temporary file
        STATE_TMP=$(mktemp -p "$TEST_DIR/integration/state/")
        
        # Capture pre-mv permissions of the temporary file
        PRE_PERMS=$(stat -c "%a" "$STATE_TMP")
        
        # Iteration 10 Worker used mv -T
        mv -T "$artifact_path" "$STATE_TMP"
        
        # Atomic rename to final destination
        mv "$STATE_TMP" "$TEST_DIR/integration/state/forge_status_123.json"
        
        # Capture post-mv permissions
        POST_PERMS=$(stat -c "%a" "$TEST_DIR/integration/state/forge_status_123.json")
        
        echo "Pre-mv temp file perms: $PRE_PERMS (expected 600)"
        echo "Final artifact perms: $POST_PERMS"
        
        if [ "$POST_PERMS" == "777" ]; then
            echo "FAIL: Permission leak detected! mv -T preserved the untrusted 777 permissions."
        else
            echo "PASS: Permissions were sanitized."
        fi
    fi
fi

# Cleanup
rm -rf "$TEST_DIR"
