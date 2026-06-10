#!/bin/bash
set -ouex pipefail

echo "--- Installing Desktop Environment ---"

# Install Niri e dipendenze cursori
dnf -y install --setopt=install_weak_deps=False niri bibata-cursor-theme

# Install Greetd e Tuigreet (Greeter da terminale in Rust)
dnf -y install --setopt=install_weak_deps=False greetd tuigreet

# Abilita i COPR per l'ecosistema Rust Wayland e installa i pacchetti
# Anyrun: App Launcher iper-veloce in Rust
dnf -y copr enable anyrun-org/anyrun
dnf -y install --setopt=install_weak_deps=False anyrun

# Ironbar: Status bar moderna e configurabile scritta in Rust
dnf -y copr enable sneexy/ironbar
dnf -y install --setopt=install_weak_deps=False ironbar
