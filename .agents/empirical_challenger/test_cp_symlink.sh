#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)
ln -s /etc/passwd "$TMP_WORKSPACE/symlink"

# Copy using cp -T
cp -T "$TMP_WORKSPACE/symlink" "$TMP_WORKSPACE/dest"

if [ -L "$TMP_WORKSPACE/dest" ]; then
    echo "SAFE: cp -T copied the symlink itself."
else
    echo "VULNERABLE: cp -T copied the contents!"
    head -n 2 "$TMP_WORKSPACE/dest"
fi

rm -rf "$TMP_WORKSPACE"
