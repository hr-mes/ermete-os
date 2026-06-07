#!/bin/bash
set -ouex pipefail

echo "--- Installing Desktop Environment ---"

# Install Niri 
dnf -y install niri bibata-cursor-theme

# FIX: Utilizzo nativo e crittograficamente sicuro del plugin COPR di DNF. 
# Previene l'iniezione MITM di archivi repo corrotti e abilita la verifica GPG hardcoded.
dnf -y copr enable avengemedia/dms

# Install Dank Linux shell
dnf -y install quickshell dms greetd dms-greeter --allowerasing
