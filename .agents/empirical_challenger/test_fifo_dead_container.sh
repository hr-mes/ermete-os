#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)

# Attacker creates FIFO and exits
podman run --rm -v "$TMP_WORKSPACE:$TMP_WORKSPACE:Z" alpine mkfifo "$TMP_WORKSPACE/artifact.json"

# Daemon sequential execution
resolved_path=$(realpath "$TMP_WORKSPACE/artifact.json")
if [ -f "$resolved_path" ]; then
    echo "FAILED: [ -f ] allowed a FIFO"
else
    echo "PASSED: [ -f ] rejected a FIFO when container is dead"
fi

rm -rf "$TMP_WORKSPACE"
