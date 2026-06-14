#!/bin/bash
set -ouex pipefail

echo "--- Applying Private Power-User Optimizations ---"

# 1. Installazione dipendenze
dnf -y install --setopt=install_weak_deps=False greenboot greenboot-default-health-checks

# Configurazione Greenboot, fwupd, First-boot Service e Bootc kargs
# Tutte queste configurazioni e unit systemd sono state migrate nativamente su /system_files/
# garantendo un design architetturale OCI dichiarativo e pulito.

# Ripristina permessi di esecuzione per gli script migrati nativamente
# (Deprecato: I permessi +x sono ereditati nativamente dalle ACL di Git)

echo "--- Private Optimizations Applied ---"
