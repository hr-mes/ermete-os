#!/bin/bash
# test_trap_failure.sh
# Tests the trap failure caused by removing conditional checks and unshare.

# Worker's trap logic
trap 'podman rm -v -f $(cat "$CIDFILE")' EXIT

# Simulate script crash BEFORE CIDFILE is created
CIDFILE="/tmp/nonexistent_cidfile_$$"

# Crash
echo "Simulating crash before podman run..."
exit 1

# Verification: When executed, this script will output:
# cat: /tmp/nonexistent_cidfile_XXXX: No such file or directory
# podman rm: requires at least 1 argument
# The trap fails and exits, stranding any other cleanup steps!
