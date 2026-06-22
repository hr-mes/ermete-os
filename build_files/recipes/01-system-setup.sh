#!/bin/bash
set -ouex pipefail

echo "--- Configuring DNF and installing base system packages ---"

# Approccio Idempotente e Cloud-Native per configurare DNF
# Ereditato nativamente da /system_files/usr/lib/dnf/dnf.conf.d/99-parallel-downloads.conf

# System apps & Dipendenze Core
# Aggiunto nix (package manager funzionale immutabile) per container e dipendenze utente
# Libvirt e virt-manager mantenuti per workflow utente quotidiano
# Aggiunti greenboot e greenboot-default-health-checks consolidati dagli script deprecati

dnf5 -y install --setopt=install_weak_deps=False libvirt virt-manager qemu-kvm sysstat lxqt-openssh-askpass parallel just nix greenboot greenboot-default-health-checks bpftool drm_info nftables wayland-utils firewalld btrfs-progs

# Implementazione dell'Hack Nix per OSTree (Salvataggio Layer Iniziale)
# Spostiamo il contenuto dello stato di Nix (/nix/var) in una directory statica del rootfs.
# Al primo avvio, un servizio systemd o tmpfiles.d lo copierà nel mountpoint dinamico (/var/opt/nix/var).
mkdir -p /usr/share/nix-initial-state
mv /nix/var /usr/share/nix-initial-state/ || true

# Rimuovi il file tmpfiles.d nativo del demone Nix per evitare conflitti (Read-Only FS)
# Gestito nativamente tramite override in /system_files/usr/lib/tmpfiles.d/nix-daemon.conf
> /usr/lib/tmpfiles.d/nix-daemon.conf || true
> /usr/lib/tmpfiles.d/nix.conf || true

# Elimina errori tmpfiles duplicati per Nix in provision.conf
sed -i '\|d /nix/var|d' /usr/lib/tmpfiles.d/provision.conf || true

# Mask NetworkManager-wait-online.service (Zero-Boot-Delay)
ln -sf /dev/null /usr/lib/systemd/system/NetworkManager-wait-online.service

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
systemd-sysusers || true

# Applicazione Hardening UNIX su /etc/skel è demandata allo stage build-symlinks nel Containerfile per purezza OCI

echo "--- Disabling fingerprint auth to fix missing pam_fprintd.so ---"
authselect disable-feature with-fingerprint || true
authselect apply-changes || true
