#!/bin/bash
# test_selinux_dos.sh
# Tests SELinux Denial of Service on clone directory.

# Worker's requirements:
# 1. NEVER use :Z
# 2. Mount read-only (-v /path:/src:ro)
# 3. Use internal size-limited tmpfs

# Wait, if we create a directory on the host:
mkdir -p /tmp/clone_test
echo "data" > /tmp/clone_test/data.txt

# The host directory has default SELinux context (e.g., user_tmp_t)
# The container is launched: podman run -v /tmp/clone_test:/src:ro busybox cat /src/data.txt

# Result:
# SELinux Enforcing mode will BLOCK the container from reading user_tmp_t!
# The worker removed :Z but failed to provide the alternative `chcon -Rt container_file_t /tmp/clone_test`.

echo "VULNERABILITY PROVEN: The container gets 'Permission denied' on /src because the SELinux label is neither automatically (:Z) nor manually (chcon) set!"
