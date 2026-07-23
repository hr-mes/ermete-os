#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)
mkdir "$TMP_WORKSPACE/target"
touch "$TMP_WORKSPACE/target/file"

# Create a symlink
ln -s "$TMP_WORKSPACE/target" "$TMP_WORKSPACE/symlink"

# Run rm -rf on the symlink
rm -rf "$TMP_WORKSPACE/symlink"

if [ -f "$TMP_WORKSPACE/target/file" ]; then
    echo "SAFE: rm -rf deleted the symlink, not the target."
else
    echo "VULNERABLE: rm -rf followed the symlink and deleted the target."
fi

rm -rf "$TMP_WORKSPACE"
