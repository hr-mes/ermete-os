#!/bin/bash
set -ouex pipefail

echo "--- Cleaning up image ---"

# Clean up dnf cache to reduce image size
dnf5 -y clean all
rm -rf /run/dnf
rm -rf /var/lib/dnf
