#!/bin/bash
set -ouex pipefail

echo "--- Configuring DNF and installing base system packages ---"

# Approccio Idempotente e Cloud-Native per configurare DNF
mkdir -p /etc/dnf/dnf.conf.d/
cat > /etc/dnf/dnf.conf.d/99-parallel-downloads.conf << EOF
[main]
max_parallel_downloads=10
EOF

# System apps
dnf -y install libvirt virt-manager qemu-kvm flatpak-builder wlr-randr iotop sysstat lxqt-openssh-askpass lxpolkit parallel just seahorse

# User apps
dnf -y install nautilus kitty mpv gnome-terminal gnome-system-monitor gnome-calculator
