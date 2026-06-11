#!/bin/bash
set -ouex pipefail

echo "--- Installing Desktop Environment ---"

# Install Niri e dipendenze cursori, temi e font
dnf -y install --setopt=install_weak_deps=False niri \
    papirus-icon-theme adw-gtk3-theme jetbrains-mono-fonts rsms-inter-fonts fontawesome-fonts-all \
    xdg-desktop-portal-gnome xdg-desktop-portal-gtk || true

# Installazione manuale sicura di Bibata Cursor (pinned version e checksum)
mkdir -p /usr/share/icons
cd /tmp
curl -sLO https://github.com/ful1e5/Bibata_Cursor/releases/download/v2.0.7/Bibata-Modern-Classic.tar.xz
echo "7d3495864e5bbef02f5e77de760b2905903b63c71495a78ef6306d19a3b556d8  Bibata-Modern-Classic.tar.xz" | sha256sum -c -
tar -xJ -C /usr/share/icons/ -f Bibata-Modern-Classic.tar.xz
rm -f Bibata-Modern-Classic.tar.xz
cd /

# Install Greetd e Tuigreet (Greeter da terminale in Rust)
dnf -y install --setopt=install_weak_deps=False greetd tuigreet

# Compilazione nativa in Rust per Anyrun e Ironbar
# Evitiamo repository COPR inaffidabili o non aggiornati per Rawhide (fc43).
# Installiamo i tool di compilazione e le dipendenze GTK/Wayland necessarie.
dnf -y install --setopt=install_weak_deps=False rust cargo gcc gcc-c++ pkgconf-pkg-config \
    glib2-devel gtk3-devel gtk4-devel gtk-layer-shell-devel gtk4-layer-shell-devel \
    cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel \
    autoconf automake libtool libevdev-devel upower-devel pulseaudio-libs-devel \
    libxkbcommon-devel wayland-devel openssl-devel luajit-devel clang \
    libinput-devel wayland-protocols-devel dbus-devel

# Compilazione e installazione
export CARGO_HOME=/tmp/cargo
cargo install anyrun ironbar starship bottom

# Sposta i binari in una directory di sistema globale
mv /tmp/cargo/bin/anyrun /usr/bin/
mv /tmp/cargo/bin/ironbar /usr/bin/
mv /tmp/cargo/bin/starship /usr/bin/
mv /tmp/cargo/bin/btm /usr/bin/

# Pulizia: rimuoviamo i tool di build per non appesantire l'immagine OCI atomica
dnf -y remove rust cargo gcc gcc-c++ \
    glib2-devel gtk3-devel gtk4-devel gtk-layer-shell-devel gtk4-layer-shell-devel \
    cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel \
    autoconf automake libtool libevdev-devel upower-devel pulseaudio-libs-devel \
    libxkbcommon-devel wayland-devel openssl-devel luajit-devel clang \
    libinput-devel wayland-protocols-devel dbus-devel
rm -rf /tmp/cargo
