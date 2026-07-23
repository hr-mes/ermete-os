#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)

# Attacker creates symlink and exits
ln -s /etc/passwd "$TMP_WORKSPACE/artifact.json"

# Daemon sequential execution
resolved_path=$(realpath "$TMP_WORKSPACE/artifact.json")
if [[ "$resolved_path" == "$TMP_WORKSPACE/"* ]]; then
    echo "FAILED: Prefix check allowed symlink outside workspace!"
else
    echo "PASSED: Prefix check successfully blocked symlink outside workspace."
fi

rm -rf "$TMP_WORKSPACE"
