#!/bin/bash
set -e

# Setup mock workspace
TMP_WORKSPACE=$(mktemp -d)
touch "$TMP_WORKSPACE/artifact.json"

# Attacker process running in background replacing file with symlink constantly
while true; do
    rm -f "$TMP_WORKSPACE/artifact.json"
    ln -s /etc/passwd "$TMP_WORKSPACE/artifact.json"
    rm -f "$TMP_WORKSPACE/artifact.json"
    touch "$TMP_WORKSPACE/artifact.json"
done &
ATTACKER_PID=$!

# Daemon simulated execution
for i in {1..100}; do
    resolved_path=$(realpath "$TMP_WORKSPACE/artifact.json" 2>/dev/null || echo "")
    if [[ "$resolved_path" == "$TMP_WORKSPACE/"* || "$resolved_path" == "$TMP_WORKSPACE"/* || "$resolved_path" == "$TMP_WORKSPACE" ]]; then
        if [ -f "$resolved_path" ]; then
            content=$(head -n 1 "$resolved_path" 2>/dev/null || true)
            if [[ "$content" == *"root:x:0:0"* ]]; then
                echo "VULNERABILITY FOUND: Arbitrary file read achieved at iteration $i!"
                kill $ATTACKER_PID
                rm -rf "$TMP_WORKSPACE"
                exit 1
            fi
        fi
    fi
done

echo "No vulnerability found in 100 iterations."
kill $ATTACKER_PID
rm -rf "$TMP_WORKSPACE"
