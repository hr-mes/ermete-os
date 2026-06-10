#!/bin/bash
set -ouex pipefail

echo "--- Installing Desktop Environment ---"

# Install Niri e dipendenze cursori, temi e font
dnf -y install --setopt=install_weak_deps=False niri bibata-cursor-theme \
    papirus-icon-theme adw-gtk3-theme jetbrains-mono-fonts rsms-inter-fonts fontawesome-fonts-all

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
cargo install anyrun
cargo install ironbar

# Sposta i binari in una directory di sistema globale
mv /root/.cargo/bin/anyrun /usr/bin/
mv /root/.cargo/bin/ironbar /usr/bin/

# Pulizia: rimuoviamo i tool di build per non appesantire l'immagine OCI atomica
dnf -y remove rust cargo gcc gcc-c++ \
    glib2-devel gtk3-devel gtk4-devel gtk-layer-shell-devel gtk4-layer-shell-devel \
    cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel \
    autoconf automake libtool libevdev-devel upower-devel pulseaudio-libs-devel \
    libxkbcommon-devel wayland-devel openssl-devel luajit-devel clang \
    libinput-devel wayland-protocols-devel dbus-devel
rm -rf /root/.cargo
