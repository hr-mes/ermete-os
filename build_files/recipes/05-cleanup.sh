#!/bin/bash
set -ouex pipefail

echo "--- Cleaning up image ---"

# Clean up dnf cache to reduce image size
# FIX: Sostituzione radicale dei comandi imperativi rm -rf che causavano la destrutturazione
# silente del bootc lint e paralizzavano l'algoritmo di relabeling di SELinux MAC.
dnf -y clean all
if command -v dnf5 >/dev/null 2>&1; then dnf5 -y clean all; fi

# Azzeramento del Machine ID per garantire la privacy su cloni multipli
# systemd genererà un ID univoco e casuale al primo avvio
truncate -s 0 /etc/machine-id
