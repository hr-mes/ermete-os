#!/bin/bash
set -ouex pipefail

echo "--- Installing Desktop Environment ---"

# Enforce System-wide Dark Mode & Aesthetics for GTK4/GTK3 native apps
echo "--- Applicazione Forzata del Tema Globale GTK ---"
mkdir -p /usr/share/glib-2.0/schemas/
cat > /usr/share/glib-2.0/schemas/99-ermete.gschema.override << 'EOF'
[org.gnome.desktop.interface]
color-scheme='prefer-dark'
gtk-theme='adw-gtk3-dark'
icon-theme='Papirus-Dark'
cursor-theme='Bibata-Modern-Classic'
EOF
glib-compile-schemas /usr/share/glib-2.0/schemas/

# Install Niri e dipendenze cursori, temi e font (aggiunto Xwayland per compatibilità assoluta con vecchie app)
dnf -y install --setopt=install_weak_deps=False niri xorg-x11-server-Xwayland \
    papirus-icon-theme adw-gtk3-theme jetbrains-mono-fonts rsms-inter-fonts fontawesome-fonts-all \
    xdg-desktop-portal-gnome xdg-desktop-portal-gtk swaybg gtk-layer-shell || true

# Configurazione Ambiente Wayland/NVIDIA (Miglioramento UX)
cat >> /etc/environment << 'EOF'
MOZ_ENABLE_WAYLAND=1
ELECTRON_OZONE_PLATFORM_HINT=wayland
NVD_BACKEND=direct
LIBVA_DRIVER_NAME=nvidia
__GLX_VENDOR_LIBRARY_NAME=nvidia
EOF

# Creazione del wrapper di sessione obbligatorio per Niri
cat > /usr/bin/niri-session << 'EOF'
#!/bin/bash
# Wrapper di sessione: innesca l'albero DBus, XDG-Desktop-Portal e Pipewire
export XDG_SESSION_TYPE=wayland
export XDG_CURRENT_DESKTOP=niri
export XDG_SESSION_DESKTOP=niri

# Importa le variabili nel manager systemd --user e nell'ambiente DBus
systemctl --user import-environment XDG_SESSION_TYPE XDG_CURRENT_DESKTOP XDG_SESSION_DESKTOP
dbus-update-activation-environment --systemd XDG_SESSION_TYPE XDG_CURRENT_DESKTOP XDG_SESSION_DESKTOP

# Attesa asincrona per i nodi DRM NVIDIA (tolleranza logind/udev)
while [ ! -e /dev/dri/renderD128 ]; do sleep 0.5; done

# Mantra Infrangibile: Avvio supervisionato via systemd-cat e dbus user session
exec systemd-cat -t niri niri --session
EOF
chmod +x /usr/bin/niri-session


# I binari (Ironbar, Starship, Bottom, Anyrun) sono compilati nativamente 
# ed esportati nel filesystem finale grazie all'architettura Multi-Stage OCI
# del Containerfile, che garantisce caching estremo e purezza del layer.

# Installazione manuale sicura di Bibata Cursor (pinned version)
mkdir -p /usr/share/icons
cd /tmp
curl -sLO "https://github.com/ful1e5/Bibata_Cursor/releases/download/${BIBATA_VER}/Bibata-Modern-Classic.tar.xz"
tar -xJ --no-same-owner -C /usr/share/icons/ -f Bibata-Modern-Classic.tar.xz
rm -f Bibata-Modern-Classic.tar.xz
cd /

# Install Greetd e Tuigreet (Greeter da terminale in Rust)
dnf -y install --setopt=install_weak_deps=False greetd tuigreet fprintd-pam
systemctl enable greetd.service

# Abilitazione Globale Audio Pipewire per la sessione utente (Fondamentale per Wayland/Portals)
systemctl --global enable pipewire.socket pipewire.service wireplumber.service

# Abilita i servizi Systemd User asincroni per componenti Wayland
systemctl --global enable ironbar.service swaybg.service || true

