#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)
ln -s loop1 "$TMP_WORKSPACE/loop2"
ln -s loop2 "$TMP_WORKSPACE/loop1"

# Daemon sequential execution
resolved_path=$(realpath "$TMP_WORKSPACE/loop1" 2>/dev/null || echo "FAILED_REALPATH")
if [ "$resolved_path" = "FAILED_REALPATH" ]; then
    echo "SAFE: realpath failed on symlink loop"
else
    echo "VULNERABLE? resolved to $resolved_path"
fi
rm -rf "$TMP_WORKSPACE"
