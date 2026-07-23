#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)
mkfifo "$TMP_WORKSPACE/artifact.json"
resolved_path=$(realpath "$TMP_WORKSPACE/artifact.json")
if [ -f "$resolved_path" ]; then
    echo "FAILED: [ -f ] allowed a FIFO"
else
    echo "PASSED: [ -f ] rejected a FIFO"
fi
rm -rf "$TMP_WORKSPACE"
