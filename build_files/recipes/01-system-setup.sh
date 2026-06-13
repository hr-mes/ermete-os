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

dnf -y install --setopt=install_weak_deps=False libvirt virt-manager qemu-kvm wlr-randr sysstat lxqt-openssh-askpass lxpolkit parallel just seahorse gnome-keyring network-manager-applet blueman playerctl brightnessctl alacritty
dnf -y install --setopt=install_weak_deps=False swaylock # Dipendenza critica per il blocco schermo di Niri

# Core Utilities in Rust (Il nuovo stack)
dnf -y install --setopt=install_weak_deps=False eza bat fd-find ripgrep nushell neovim ananicy-cpp

# Installazione di Starship e Bottom ora delegata al processo nativo Cargo (vedi 03-desktop.sh)

# Fix Missing UNIX Groups (Wayland/DRM/Audio/Input)
# In container builds, sometimes standard groups from 'setup' rpm are not fully populated.
# We explicitly create the critical ones to prevent udev permission failures (schermo nero post-login).
echo "--- Fixing critical UNIX groups for Wayland/udev ---"
for g in video render input tty audio kvm disk lp clock sgx kmem tss; do
    getent group $g >/dev/null || groupadd -r $g || true
done

# Inneschiamo esplicitamente sysusers per assicurarci che anche systemd applichi i suoi default
systemd-sysusers || true
