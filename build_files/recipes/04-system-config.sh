#!/bin/bash
set -ouex pipefail

echo "--- Configuring system services and user defaults ---"

# Install greetd login manager with dank configuration
mkdir -p /etc/greetd/
cat > /etc/greetd/config.toml << EOF
[terminal]
vt = 1
[default_session]
user = "greeter"
command = "dms-greeter --command niri"
EOF

# Set Greetd as default display manager (Approccio nativo Systemd)
systemctl enable greetd.service

# Setup DMS service for all new users (Abilitazione globale)
systemctl --global enable dms.service

# Copy Niri dotfiles to skel
mkdir -p /etc/skel/.config/niri/
cp -rf /ctx/dot_config/niri/config.kdl /etc/skel/.config/niri/

#### Enable podman
systemctl enable podman.socket

# Remove waybar
dnf -y remove waybar
