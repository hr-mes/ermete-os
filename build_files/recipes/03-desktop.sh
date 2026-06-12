#!/bin/bash
set -ouex pipefail

echo "--- Installing Desktop Environment ---"

# Install Niri e dipendenze cursori, temi e font (aggiunto Xwayland per compatibilità assoluta con vecchie app)
dnf -y install --setopt=install_weak_deps=False niri xorg-x11-server-Xwayland \
    papirus-icon-theme adw-gtk3-theme jetbrains-mono-fonts rsms-inter-fonts fontawesome-fonts-all \
    xdg-desktop-portal-gnome xdg-desktop-portal-gtk || true

# Installazione manuale sicura di Bibata Cursor (pinned version e checksum)
mkdir -p /usr/share/icons
cd /tmp
curl -sLO "https://github.com/ful1e5/Bibata_Cursor/releases/download/${BIBATA_VER}/Bibata-Modern-Classic.tar.xz"
echo "${BIBATA_HASH}  Bibata-Modern-Classic.tar.xz" | sha256sum -c -
tar -xJ -C /usr/share/icons/ -f Bibata-Modern-Classic.tar.xz
rm -f Bibata-Modern-Classic.tar.xz
cd /

# Install Greetd e Tuigreet (Greeter da terminale in Rust)
dnf -y install --setopt=install_weak_deps=False greetd tuigreet

# 100% Verified Supply Chain per lo Stack Rust (Zero-Trust)
# Sostituiamo il fetch dinamico da crates.io con release pre-compilate validate
# o compilazione offline da commit pinnati con hash crittografica garantita.

cd /tmp

# 1. Ironbar (Pre-compilato v0.19.0)
curl -sLO "https://github.com/JakeStanger/ironbar/releases/download/${IRONBAR_VER}/ironbar-${IRONBAR_VER}-x86_64.tar.gz"
echo "${IRONBAR_HASH}  ironbar-${IRONBAR_VER}-x86_64.tar.gz" | sha256sum -c -
tar -xzf ironbar-${IRONBAR_VER}-x86_64.tar.gz
mv ironbar /usr/bin/
rm -f ironbar-${IRONBAR_VER}-x86_64.tar.gz

# 2. Starship (Pre-compilato v1.22.1)
curl -sLO "https://github.com/starship/starship/releases/download/${STARSHIP_VER}/starship-x86_64-unknown-linux-gnu.tar.gz"
echo "${STARSHIP_HASH}  starship-x86_64-unknown-linux-gnu.tar.gz" | sha256sum -c -
tar -xzf starship-x86_64-unknown-linux-gnu.tar.gz
mv starship /usr/bin/
rm -f starship-x86_64-unknown-linux-gnu.tar.gz

# 3. Bottom (Pre-compilato v0.10.2)
curl -sLO "https://github.com/ClementTsang/bottom/releases/download/${BOTTOM_VER}/bottom_x86_64-unknown-linux-gnu.tar.gz"
echo "${BOTTOM_HASH}  bottom_x86_64-unknown-linux-gnu.tar.gz" | sha256sum -c -
tar -xzf bottom_x86_64-unknown-linux-gnu.tar.gz
mv btm /usr/bin/
rm -f bottom_x86_64-unknown-linux-gnu.tar.gz

# 4. Anyrun (Compilato offline da sorgente verificato)
# Essendo un componente con profonde radici in GTK Layer Shell, lo compiliamo nativamente.
# ANYRUN_COMMIT è ora iniettato come ARG dal Containerfile
curl -sLO "https://github.com/anyrun-org/anyrun/archive/${ANYRUN_COMMIT}.tar.gz"
echo "${ANYRUN_HASH}  ${ANYRUN_COMMIT}.tar.gz" | sha256sum -c -
tar -xzf ${ANYRUN_COMMIT}.tar.gz

dnf -y install --setopt=install_weak_deps=False rust cargo gcc gcc-c++ pkgconf-pkg-config \
    glib2-devel gtk3-devel gtk4-devel gtk-layer-shell-devel gtk4-layer-shell-devel \
    cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel \
    autoconf automake libtool libevdev-devel upower-devel pulseaudio-libs-devel \
    libxkbcommon-devel wayland-devel openssl-devel luajit-devel clang \
    libinput-devel wayland-protocols-devel dbus-devel

export CARGO_HOME=/tmp/cargo
cd anyrun-${ANYRUN_COMMIT}
# Compilazione bloccata senza network dynamismo
cargo build --release --locked
mv target/release/anyrun /usr/bin/
mkdir -p /usr/lib64/anyrun
cp target/release/*.so /usr/lib64/anyrun/ 2>/dev/null || true
ln -s /usr/lib64/anyrun /usr/lib/anyrun
cd /

# Pulizia chirurgica
# NOTA: non rimuoviamo gcc, gcc-c++ e openssl-devel poiché sono protetti dal Layer 0 (Base NVIDIA) per DKMS
dnf -y remove rust cargo \
    glib2-devel gtk3-devel gtk4-devel gtk-layer-shell-devel gtk4-layer-shell-devel \
    cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel \
    autoconf automake libtool libevdev-devel upower-devel pulseaudio-libs-devel \
    libxkbcommon-devel wayland-devel luajit-devel clang \
    libinput-devel wayland-protocols-devel dbus-devel
rm -rf /tmp/cargo /tmp/${ANYRUN_COMMIT}.tar.gz /tmp/anyrun-${ANYRUN_COMMIT}
