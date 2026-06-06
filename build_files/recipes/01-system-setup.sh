#!/bin/bash
set -ouex pipefail

echo "--- Configuring DNF and installing base system packages ---"
sed -i '/^\[main\]/a max_parallel_downloads=9' /etc/dnf/dnf.conf

# System apps
dnf -y install libvirt virt-manager qemu-kvm flatpak-builder wlr-randr iotop sysstat lxqt-openssh-askpass lxpolkit parallel just seahorse

# User apps
dnf -y install nautilus kitty mpv gnome-terminal gnome-system-monitor gnome-calculator
