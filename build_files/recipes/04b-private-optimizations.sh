#!/bin/bash
set -ouex pipefail

echo "--- Applying Private Power-User Optimizations ---"

# 1. Installazione dipendenze
dnf -y install --setopt=install_weak_deps=False greenboot greenboot-default-health-checks

# Configurazione Greenboot, fwupd, First-boot Service e Bootc kargs
# Tutte queste configurazioni e unit systemd sono state migrate nativamente su /system_files/
# garantendo un design architetturale OCI dichiarativo e pulito.

# Ripristina permessi di esecuzione per gli script migrati nativamente
chmod +x /etc/greenboot/check/required.d/*.sh /usr/libexec/ermete-firstboot.sh || true

echo "--- Private Optimizations Applied ---"
