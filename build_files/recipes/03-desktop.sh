#!/bin/bash
set -ouex pipefail

echo "--- Installing Desktop Environment ---"

# Enforce System-wide Dark Mode & Aesthetics for GTK4/GTK3 native apps
echo "--- Applicazione Forzata del Tema Globale GTK ---"
# I file .gschema.override sono ereditati staticamente da /system_files/
glib-compile-schemas /usr/share/glib-2.0/schemas/

# Install Niri e dipendenze cursori, temi e font (aggiunto Xwayland per compatibilità assoluta con vecchie app)
dnf -y install --setopt=install_weak_deps=False niri xorg-x11-server-Xwayland \
    papirus-icon-theme adw-gtk3-theme jetbrains-mono-fonts rsms-inter-fonts fontawesome-fonts-all \
    xdg-desktop-portal-gnome xdg-desktop-portal-gtk swaybg gtk-layer-shell || true

# Configurazione Ambiente Wayland/NVIDIA e wrapper niri-session 
# sono ereditati nativamente da /system_files/etc/environment e /system_files/usr/bin/niri-session
chmod +x /usr/bin/niri-session


# I binari (Ironbar, Starship, Bottom, Anyrun) sono compilati nativamente 
# ed esportati nel filesystem finale grazie all'architettura Multi-Stage OCI
# del Containerfile, che garantisce caching estremo e purezza del layer.

# Installazione manuale sicura di Bibata Cursor (pinned version)
# Ereditata nativamente dal builder multi-stage OCI (Zero-Network-Failure)

# Install Greetd e Tuigreet (Greeter da terminale in Rust)
dnf -y install --setopt=install_weak_deps=False greetd tuigreet
# Greetd service è abilitato nativamente via system-preset

# Abilitazione Globale Audio Pipewire per la sessione utente (Fondamentale per Wayland/Portals)
# Abilitazione Globale Audio Pipewire e Wayland User Services
# I servizi sono abilitati nativamente via OCI tramite preset in /usr/lib/systemd/user-preset/
