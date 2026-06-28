#!/bin/bash
set -ouex pipefail

echo "--- Configuring DNF and installing base system packages ---"

# Approccio Idempotente e Cloud-Native per configurare DNF
# Ereditato nativamente da /system_files/usr/lib/dnf/dnf.conf.d/99-parallel-downloads.conf

# System apps & Dipendenze Core
# Libvirt e virt-manager mantenuti per workflow utente quotidiano
# Aggiunti greenboot e greenboot-default-health-checks consolidati dagli script deprecati

# Core Utilities in Rust (Il nuovo stack)
dnf5 -y install --setopt=install_weak_deps=False eza bat fd-find ripgrep nushell neovim ananicy-cpp

# System tools, hypervisor, firewall
dnf5 -y install --setopt=install_weak_deps=False libvirt virt-manager qemu-kvm sysstat parallel just greenboot greenboot-default-health-checks bpftool drm_info nftables wayland-utils firewalld btrfs-progs

# Implementazione dell'Hack Nix per OSTree (Salvataggio Layer Iniziale)
# Spostiamo il contenuto dello stato di Nix (/nix/var) in una directory statica del rootfs.
# Al primo avvio, un servizio systemd o tmpfiles.d lo copierà nel mountpoint dinamico (/var/opt/nix/var).
mkdir -p /usr/share/nix-initial-state
mv /nix/var /usr/share/nix-initial-state/ || true



# Hardening Networking (Zero-Trust)

# I servizi firewalld e ananicy-cpp sono abilitati nativamente via system-preset

echo "--- Applicazione Forzata Firewalld Default Zone ---"
# Applica il drop nativo e idempotente senza sovrascrivere l'intero firewalld.conf
firewall-offline-cmd --set-default-zone=drop
firewall-offline-cmd --zone=drop --add-service=mdns

# Installazione di Starship via Cargo, Btop via DNF (vedi 03-desktop.sh)

# Fix Missing UNIX Groups (Wayland/DRM/Audio/Input)
# In container builds, sometimes standard groups from 'setup' rpm are not fully populated.
# L'approccio imperativo (groupadd) è stato rimosso per rispettare l'immutabilità OCI.
# Tutti i gruppi vitali sono ora definiti in modo puramente deterministico 
# tramite /usr/lib/sysusers.d/10-ermete-hw-groups.conf (ereditato dal Layer 0).
echo "--- Fixing critical UNIX groups for Wayland/udev via sysusers ---"

# Inneschiamo esplicitamente sysusers per assicurarci che anche systemd applichi i suoi default
systemd-sysusers || true

# Applicazione Hardening UNIX su /etc/skel è demandata allo stage build-symlinks nel Containerfile per purezza OCI


