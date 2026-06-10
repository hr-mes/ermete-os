#!/bin/bash
set -ouex pipefail

echo "--- Installing Desktop Environment ---"

# Install Niri e dipendenze cursori
dnf -y install --setopt=install_weak_deps=False niri bibata-cursor-theme

# Install Greetd e Tuigreet (Greeter da terminale in Rust)
dnf -y install --setopt=install_weak_deps=False greetd tuigreet

# Anyrun: App Launcher iper-veloce in Rust
cat > /etc/yum.repos.d/_copr:copr.fedorainfracloud.org:reisaraujo-miguel:Anyrun.repo << 'EOF'
[copr:copr.fedorainfracloud.org:reisaraujo-miguel:Anyrun]
name=Copr repo for Anyrun owned by reisaraujo-miguel
baseurl=https://download.copr.fedorainfracloud.org/results/reisaraujo-miguel/Anyrun/fedora-43-$basearch/
type=rpm-md
skip_if_unavailable=True
gpgcheck=1
gpgkey=https://download.copr.fedorainfracloud.org/results/reisaraujo-miguel/Anyrun/pubkey.gpg
repo_gpgcheck=0
enabled=1
enabled_metadata=1
EOF
dnf -y install --setopt=install_weak_deps=False Anyrun

# Ironbar: Status bar moderna e configurabile scritta in Rust (forzato su fedora-42)
cat > /etc/yum.repos.d/_copr:copr.fedorainfracloud.org:deluxe-cube:ironbar.repo << 'EOF'
[copr:copr.fedorainfracloud.org:deluxe-cube:ironbar]
name=Copr repo for ironbar owned by deluxe-cube
baseurl=https://download.copr.fedorainfracloud.org/results/deluxe-cube/ironbar/fedora-42-$basearch/
type=rpm-md
skip_if_unavailable=True
gpgcheck=1
gpgkey=https://download.copr.fedorainfracloud.org/results/deluxe-cube/ironbar/pubkey.gpg
repo_gpgcheck=0
enabled=1
enabled_metadata=1
EOF
dnf -y install --setopt=install_weak_deps=False ironbar
