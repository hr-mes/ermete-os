#!/bin/bash
set -ouex pipefail

echo "--- Installing Full-Stack Rust Desktop Environment ---"

# 1. Abilitazione repository COPR per i componenti grafici scritti in Rust
dnf -y copr enable psoldunov/regreet
dnf -y copr enable victorvintorez/packages # Contiene ironbar-git
dnf -y copr enable reisaraujo-miguel/Anyrun
dnf -y copr enable jvssdev/waylock
dnf -y copr enable agonie/swww

# 2. Installazione dello stack grafico nativo Rust
dnf -y install \
    niri \
    alacritty \
    yazi \
    greetd \
    regreet \
    ironbar \
    anyrun \
    waylock \
    swww \
    --allowerasing
