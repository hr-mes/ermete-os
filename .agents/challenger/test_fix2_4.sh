#!/bin/bash
set -e

echo "=== Testing Fix 2 Mitigation: chmod -R u+rwX ==="
mkdir -p test2_workspace/a/b
touch test2_workspace/a/b/file.txt
chmod 000 test2_workspace/a

chmod -R u+rwX test2_workspace 2>/dev/null || true
if rm -rf test2_workspace 2>/dev/null; then
    echo "PASS: rm -rf succeeded with u+rwX."
else
    echo "FAIL: rm -rf still failed."
    chmod -R 777 test2_workspace || true
    rm -rf test2_workspace
fi

echo ""
echo "=== Testing Fix 4: SELinux Relabeling DoS with Symlink ==="
# We will create a fake $TMP_WORKSPACE, put a symlink to another directory,
# and see if podman's :Z relabels the target of the symlink.
mkdir -p test4_workspace
mkdir -p test4_target
touch test4_target/target_file.txt
# Give it a specific label
chcon -t tmp_t test4_target/target_file.txt 2>/dev/null || true
ORIG_LABEL=$(ls -Z test4_target/target_file.txt | awk '{print $1}')

# Create symlink in workspace
ln -s $(realpath test4_target) test4_workspace/symlink

# Run podman with :Z mount
# Note: podman must be installed, if not we'll just simulate/read docs
if command -v podman >/dev/null; then
    podman run --rm -v $(realpath test4_workspace):/workspace:Z alpine echo "relabeled" 2>/dev/null || true
    NEW_LABEL=$(ls -Z test4_target/target_file.txt | awk '{print $1}')
    if [ "$ORIG_LABEL" != "$NEW_LABEL" ] && [ -n "$ORIG_LABEL" ]; then
        echo "FAIL: Symlink was followed! Target relabeled from $ORIG_LABEL to $NEW_LABEL"
    else
        echo "PASS: Symlink was NOT followed by :Z."
    fi
else
    echo "Podman not found, skipping empirical test."
fi
rm -rf test4_workspace test4_target

