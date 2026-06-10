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
# Rimosse applet GUI (nm-applet, blueman) per rispettare il Manifesto Zero-GUI-on-Host
# Aggiunto distrobox per lo sviluppo in container
dnf -y install --setopt=install_weak_deps=False libvirt virt-manager qemu-kvm wlr-randr sysstat lxqt-openssh-askpass lxpolkit parallel just seahorse playerctl brightnessctl
dnf -y install --setopt=install_weak_deps=False swaylock # Dipendenza critica per il blocco schermo di Niri

# Core Utilities in Rust (Il nuovo stack)
dnf -y install --setopt=install_weak_deps=False eza bat fd-find ripgrep bottom nushell starship
