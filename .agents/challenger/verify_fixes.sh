#!/bin/bash
set -e

echo "=== Testing Fix 1: Arbitrary File Read via symlinks ==="
mkdir -p test1
touch test1/target.txt
echo "SECRET" > test1/target.txt

# The README states: test -L <file> (to reject) and test -f <file>
# What if the directory is a symlink?
ln -s test1 test1_link
if ! test -L "test1_link/target.txt" && test -f "test1_link/target.txt"; then
    echo "FAIL: test -L test1_link/target.txt passed! (Bypass successful)"
    cat "test1_link/target.txt"
else
    echo "PASS: test -L blocked it."
fi

echo ""
echo "=== Testing Fix 2: Trap Failure via chmod ==="
mkdir -p test2_workspace
mkdir -p test2_workspace/a/b
touch test2_workspace/a/b/file.txt
# Attacker removes all permissions from the directory
chmod 000 test2_workspace/a

# The Trap runs this:
chmod -R u+w test2_workspace 2>/dev/null || true

# Can we remove it?
if rm -rf test2_workspace 2>/dev/null; then
    echo "PASS: rm -rf succeeded."
else
    echo "FAIL: rm -rf failed! Workspace leaked! (Bypass successful)"
    # Cleanup for next run
    chmod -R 777 test2_workspace || true
    rm -rf test2_workspace
fi

echo ""
echo "=== Testing Fix 3: Directory Evasion (mv -T) ==="
# Attacker creates a directory instead of a file
mkdir -p test3_workspace/status.json
touch test3_dest
if mv -T test3_workspace/status.json test3_dest 2>/dev/null; then
    echo "FAIL: mv -T succeeded for a directory! (Bypass successful)"
else
    echo "PASS: mv -T prevented directory evasion."
fi
rm -rf test3_workspace test3_dest

echo ""
echo "=== Testing Fix 5: Zombie Containers (--cidfile) ==="
mkdir -p test5_workspace
# Attacker overwrites the cidfile with --all
echo "--all" > test5_workspace/cidfile

# The host runs: podman rm -f $(cat cidfile)
# We'll use a wrapper to see what podman gets
echo '#!/bin/bash' > podman_mock.sh
echo 'echo "PODMAN_MOCK CALLED WITH: $1 $2 $3"' >> podman_mock.sh
chmod +x podman_mock.sh

./podman_mock.sh rm -f $(cat test5_workspace/cidfile)
if [ "$(cat test5_workspace/cidfile)" == "--all" ]; then
    echo "FAIL: Podman receives --all, which will delete all host containers! (Bypass successful)"
else
    echo "PASS: Safe."
fi

rm -rf test1 test1_link test5_workspace podman_mock.sh

