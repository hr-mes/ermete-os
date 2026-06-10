#!/bin/bash
set -ouex pipefail

echo "--- Installing Desktop Environment ---"

# Install Niri e dipendenze cursori
dnf -y install --setopt=install_weak_deps=False niri bibata-cursor-theme

# Install Greetd e Tuigreet (Greeter da terminale in Rust)
dnf -y install --setopt=install_weak_deps=False greetd tuigreet

# Anyrun: App Launcher iper-veloce in Rust (forzato su fedora-41 per evitare 404 su rawhide/fc43)
cat > /etc/yum.repos.d/_copr:copr.fedorainfracloud.org:anyrun-org:anyrun.repo << 'EOF'
[copr:copr.fedorainfracloud.org:anyrun-org:anyrun]
name=Copr repo for anyrun owned by anyrun-org
baseurl=https://download.copr.fedorainfracloud.org/results/anyrun-org/anyrun/fedora-41-$basearch/
type=rpm-md
skip_if_unavailable=True
gpgcheck=1
gpgkey=https://download.copr.fedorainfracloud.org/results/anyrun-org/anyrun/pubkey.gpg
repo_gpgcheck=0
enabled=1
enabled_metadata=1
EOF
dnf -y install --setopt=install_weak_deps=False anyrun

# Ironbar: Status bar moderna e configurabile scritta in Rust (forzato su fedora-41)
cat > /etc/yum.repos.d/_copr:copr.fedorainfracloud.org:sneexy:ironbar.repo << 'EOF'
[copr:copr.fedorainfracloud.org:sneexy:ironbar]
name=Copr repo for ironbar owned by sneexy
baseurl=https://download.copr.fedorainfracloud.org/results/sneexy/ironbar/fedora-41-$basearch/
type=rpm-md
skip_if_unavailable=True
gpgcheck=1
gpgkey=https://download.copr.fedorainfracloud.org/results/sneexy/ironbar/pubkey.gpg
repo_gpgcheck=0
enabled=1
enabled_metadata=1
EOF
dnf -y install --setopt=install_weak_deps=False ironbar
