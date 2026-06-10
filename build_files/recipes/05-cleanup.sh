#!/bin/bash
set -ouex pipefail

echo "--- Cleaning up image ---"

# Clean up dnf cache to reduce image size
# FIX: Sostituzione radicale dei comandi imperativi rm -rf che causavano la destrutturazione
# silente del bootc lint e paralizzavano l'algoritmo di relabeling di SELinux MAC.
dnf -y clean all
dnf5 -y clean all

# Azzeramento del Machine ID per garantire la privacy su cloni multipli
# systemd genererà un ID univoco e casuale al primo avvio
truncate -s 0 /etc/machine-id
