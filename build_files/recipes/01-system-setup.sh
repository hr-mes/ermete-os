#!/bin/bash
set -ouex pipefail

echo "--- Configuring DNF and installing base system packages ---"

# Approccio Idempotente e Cloud-Native per configurare DNF
mkdir -p /etc/dnf/dnf.conf.d/
cat > /etc/dnf/dnf.conf.d/99-parallel-downloads.conf << EOF
[main]
max_parallel_downloads=10
EOF

# System apps & Dipendenze Core
dnf -y install libvirt virt-manager qemu-kvm flatpak-builder wlr-randr sysstat lxqt-openssh-askpass lxpolkit parallel just seahorse
dnf -y install swaylock # Dipendenza critica per il blocco schermo di Niri

# Core Utilities in Rust (Il nuovo stack)
dnf -y install eza bat fd-find ripgrep bottom nushell starship

# User apps (Rust focused)
dnf -y install nautilus alacritty mpv
