#!/bin/bash
set -ouex pipefail

echo "--- Installing Desktop Environment ---"

# Enforce System-wide Dark Mode & Aesthetics for GTK4/GTK3 native apps
echo "--- Applicazione Forzata del Tema Globale GTK ---"
# I file .gschema.override sono ereditati staticamente da /system_files/
# La compilazione degli schemi è demandata alla fine dello script per includere i pacchetti successivi.

# Install Niri e dipendenze cursori, temi e font (aggiunto Xwayland per compatibilità assoluta con vecchie app)
dnf5 -y install --setopt=install_weak_deps=False niri xorg-x11-server-Xwayland \
    mesa-dri-drivers mesa-vulkan-drivers foot upower \
    mpv imv dbus-x11 dbus-tools \
    pipewire pipewire-alsa pipewire-pulseaudio wireplumber \
    playerctl brightnessctl swaylock libnotify wlr-randr \
    papirus-icon-theme adw-gtk3-theme jetbrains-mono-fonts rsms-inter-fonts fontawesome-fonts-all \
    xdg-desktop-portal-gnome xdg-desktop-portal-gtk swaybg gtk-layer-shell gtk4-layer-shell \
    qt5-qtwayland qt6-qtwayland xdg-user-dirs xdg-user-dirs-gtk \
    gnome-keyring gnome-keyring-pam wl-clipboard btop wl-mirror nodejs npm \
    thunar thunar-archive-plugin thunar-volman gvfs file-roller

# Configurazione Ambiente Wayland/NVIDIA e wrapper niri-session 
# sono ereditati nativamente da /system_files/usr/lib/environment.d/99-ermete.conf e /system_files/usr/bin/niri-session
# I permessi di esecuzione sono vincolati nativamente nell'albero Git.

# I binari (Starship, Bottom) sono compilati nativamente 
# ed esportati nel filesystem finale grazie all'architettura Multi-Stage OCI
# del Containerfile, che garantisce caching estremo e purezza del layer.

# Installazione manuale sicura di Bibata Cursor (pinned version)
# Ereditata nativamente dal builder multi-stage OCI (Zero-Network-Failure)

# Install Greetd e Tuigreet (Greeter da terminale in Rust)
dnf5 -y install --setopt=install_weak_deps=False greetd tuigreet
# Rimuoviamo la config di default del pacchetto per favorire il fallback stateless in /usr/lib/greetd/config.toml
rm -rf /etc/greetd/config.toml
# Garantiamo l'esistenza dell'utente greeter prima del boot delegandola a systemd-sysusers.
# (Vedi system_files/usr/lib/sysusers.d/20-greeter.conf per la dichiarazione)
systemd-sysusers
# Greetd service è abilitato nativamente via system-preset
# SELinux permissive per greetd è gestito nativamente dal Containerfile tramite policycoreutils temporaneo.

# Abilitazione Globale Audio Pipewire per la sessione utente (Fondamentale per Wayland/Portals)
# Abilitazione Globale Audio Pipewire e Wayland User Services
# I servizi sono abilitati nativamente via OCI tramite preset in /usr/lib/systemd/user-preset/



# Assicura che i nuovi schemi GTK installati dai pacchetti vengano precalcolati nativamente nel layer
glib-compile-schemas /usr/share/glib-2.0/schemas/

# Sanitizzazione Authselect (Rimozione phantom dependencies come fprintd riattivate da dnf)
# Forza un profilo locale standard, disabilitando la feature fingerprint se non voluta
authselect select local with-silent-lastlog with-mdns4 without-nullok --force
authselect disable-feature with-fingerprint || true
authselect apply-changes

# Pulizia chirurgica Bloatware ed app GTK ridondanti ereditate dalle build di base
dnf5 -y remove gnome-calculator gnome-system-monitor seahorse orca || true

# (Il comando dnf5 clean all è stato rimosso in quanto incompatibile con i cache mounts OCI)
