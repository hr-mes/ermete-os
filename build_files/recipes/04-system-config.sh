#!/bin/bash
set -ouex pipefail

echo "--- Configuring system services and user defaults ---"

mkdir -p /etc/greetd/


# Install greetd login manager vincolato alla configurazione restrittiva con tuigreet
mkdir -p /etc/greetd
# 1. Configurazione Greetd, Firewalld, Coredump, MAC Randomization, DoT
# Tutte queste configurazioni sono state migrate nativamente su /system_files/etc/
# garantendo un design architetturale OCI dichiarativo e pulito.
# I system-presets nativi sono definiti in /system_files/usr/lib/systemd/system-preset/99-Ermete.preset

systemctl set-default graphical.target

# Copy all dotfiles to skel
mkdir -p /etc/skel/.config/
cp -rf /ctx/dot_config/* /etc/skel/.config/

# Crea la directory degli screenshot per i nuovi utenti
mkdir -p /etc/skel/Pictures/Screenshots

# Abilita Starship (Prompt in Rust) globalmente per le shell compatibili
echo 'eval "$(starship init bash)"' > /etc/profile.d/starship.sh

# Manutenzione BTRFS e System Limits delegati a /system_files/

echo "--- System Configuration Applied ---"

# Assicura i permessi corretti per lo skeleton directory garantendo la Privacy
# senza rompere gli script (forzando +x sui file .sh)
chown -R root:root /etc/skel/
find /etc/skel -type d -exec chmod 700 {} \;
find /etc/skel -type f -exec chmod 600 {} \;
find /etc/skel -type f -name "*.sh" -exec chmod 700 {} \;

# Btrfs Auto-Snapshot for /var/home delegato a /system_files/

# Remove waybar
dnf -y remove waybar
