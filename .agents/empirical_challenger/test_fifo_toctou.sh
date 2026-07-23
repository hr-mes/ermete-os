#!/bin/bash
set -e

TMP_WORKSPACE=$(mktemp -d)
touch "$TMP_WORKSPACE/artifact.json"

# Attacker process replacing file with FIFO
while true; do
    rm -f "$TMP_WORKSPACE/artifact.json"
    mkfifo "$TMP_WORKSPACE/artifact.json"
    rm -f "$TMP_WORKSPACE/artifact.json"
    touch "$TMP_WORKSPACE/artifact.json"
done &
ATTACKER_PID=$!

# Daemon simulation
for i in {1..200}; do
    resolved_path=$(realpath "$TMP_WORKSPACE/artifact.json" 2>/dev/null || echo "")
    if [[ "$resolved_path" == "$TMP_WORKSPACE/"* || "$resolved_path" == "$TMP_WORKSPACE"/* || "$resolved_path" == "$TMP_WORKSPACE" ]]; then
        if [ -f "$resolved_path" ]; then
            # Attempt to read - we will use a timeout to detect if it blocks
            if timeout 0.5 cat "$resolved_path" > /dev/null 2>&1; then
                # read succeeded or file was empty, loop again
                :
            else
                exit_status=$?
                if [ $exit_status -eq 124 ]; then
                    echo "VULNERABILITY FOUND: FIFO deadlock achieved via TOCTOU at iteration $i!"
                    kill $ATTACKER_PID
                    rm -rf "$TMP_WORKSPACE"
                    exit 1
                fi
            fi
        fi
    fi
done

echo "No deadlock found in 200 iterations."
kill $ATTACKER_PID
rm -rf "$TMP_WORKSPACE"
