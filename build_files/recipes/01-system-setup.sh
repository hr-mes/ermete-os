#!/bin/bash
set -ouex pipefail

echo "--- Configuring DNF and installing base system packages ---"

# Approccio Idempotente e Cloud-Native per configurare DNF
# Ereditato nativamente da /system_files/usr/lib/dnf/dnf.conf.d/99-parallel-downloads.conf

# System apps & Dipendenze Core
# Aggiunto nix (package manager funzionale immutabile) per container e dipendenze utente
# Libvirt e virt-manager mantenuti per workflow utente quotidiano
# Aggiunti greenboot e greenboot-default-health-checks consolidati dagli script deprecati

dnf5 -y install --setopt=install_weak_deps=False libvirt virt-manager qemu-kvm sysstat lxqt-openssh-askpass parallel just nix greenboot bpftool drm_info nftables wayland-utils firewalld btrfs-progs

# Core Utilities in Rust (Il nuovo stack)
dnf5 -y install --setopt=install_weak_deps=False eza bat fd-find ripgrep nushell neovim ananicy-cpp

# Hardening Networking (Zero-Trust)

# I servizi firewalld e ananicy-cpp sono abilitati nativamente via system-preset

echo "--- Applicazione Forzata Firewalld Default Zone ---"
# Applica il drop nativo e idempotente senza sovrascrivere l'intero firewalld.conf
firewall-offline-cmd --set-default-zone=drop
firewall-offline-cmd --zone=drop --add-service=mdns

# Installazione di Starship e Bottom ora delegata al processo nativo Cargo (vedi 03-desktop.sh)

# Fix Missing UNIX Groups (Wayland/DRM/Audio/Input)
# In container builds, sometimes standard groups from 'setup' rpm are not fully populated.
# L'approccio imperativo (groupadd) è stato rimosso per rispettare l'immutabilità OCI.
# Tutti i gruppi vitali sono ora definiti in modo puramente deterministico 
# tramite /usr/lib/sysusers.d/10-ermete-hw-groups.conf (ereditato dal Layer 0).
echo "--- Fixing critical UNIX groups for Wayland/udev via sysusers ---"

# Inneschiamo esplicitamente sysusers per assicurarci che anche systemd applichi i suoi default
systemd-sysusers

# Applicazione Hardening UNIX su /etc/skel è demandata allo stage build-symlinks nel Containerfile per purezza OCI
