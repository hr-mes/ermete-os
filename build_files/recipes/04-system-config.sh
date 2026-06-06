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

# Set Greetd as default display manager
rm -f /etc/systemd/system/display-manager.service
ln -s /usr/lib/systemd/system/greetd.service /etc/systemd/system/display-manager.service
systemctl enable --force greetd.service

# Setup DMS service for new users
mkdir -p /etc/skel/.config/systemd/user/graphical-session.target.wants
ln -s /usr/lib/systemd/user/dms.service /etc/skel/.config/systemd/user/graphical-session.target.wants/

# Copy Niri dotfiles to skel
mkdir -p /etc/skel/.config/niri/
cp -rf /ctx/dot_config/niri/config.kdl /etc/skel/.config/niri/

#### Enable podman
systemctl enable podman.socket

# Remove waybar
dnf -y remove waybar
