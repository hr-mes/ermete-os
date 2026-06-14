#!/bin/bash
set -ouex pipefail

echo "--- Cleaning up image ---"



# Bootc inherently handles SELinux context generation natively during build.
# Azzeramento del Machine ID per garantire la privacy su cloni multipli
# systemd genererà un ID univoco e casuale al primo avvio
truncate -s 0 /etc/machine-id

echo "--- Cleanup Complete ---"
