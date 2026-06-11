#!/bin/bash
set -ouex pipefail

echo "--- Configuring DNF and installing base system packages ---"

# Approccio Idempotente e Cloud-Native per configurare DNF
mkdir -p /etc/dnf/dnf.conf.d/
cat > /etc/dnf/dnf.conf.d/99-parallel-downloads.conf << EOF
[main]
max_parallel_downloads=9
countme=0
EOF

# System apps & Dipendenze Core
# Rimosso flatpak-builder (spostato in distrobox)
# Aggiunti network-manager-applet e blueman per l'usabilità (Wi-Fi e Bluetooth)
# Aggiunto distrobox per lo sviluppo in container
# Rimuoviamo temporaneamente il symlink /nix ereditato dalla base image per permettere a DNF
# di spacchettare il pacchetto 'nix' nativo di Fedora senza conflitti (cpio: mkdir failed - File exists).
rm -f /nix

dnf -y install --setopt=install_weak_deps=False libvirt virt-manager qemu-kvm wlr-randr sysstat lxqt-openssh-askpass lxpolkit parallel just seahorse gnome-keyring network-manager-applet blueman playerctl brightnessctl nix
dnf -y install --setopt=install_weak_deps=False swaylock # Dipendenza critica per il blocco schermo di Niri

# Se il pacchetto nix ha creato la directory, spostiamo il contenuto per il provisioning a runtime
# e ripristiniamo il symlink immutabile verso /var/nix
if [ -d "/nix" ] && [ ! -L "/nix" ]; then
    mkdir -p /usr/share/nix-base
    mv /nix/* /usr/share/nix-base/ 2>/dev/null || true
    rm -rf /nix
fi
ln -s /var/nix /nix

# Core Utilities in Rust (Il nuovo stack)
dnf -y install --setopt=install_weak_deps=False eza bat fd-find ripgrep nushell neovim

# Installazione di Starship e Bottom ora delegata al processo nativo Cargo (vedi 03-desktop.sh)
