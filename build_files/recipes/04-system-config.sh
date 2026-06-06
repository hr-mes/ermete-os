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

# FIX: Architettura Systemd nativa (Systemd Presets) invece di usare systemctl enable
mkdir -p /usr/lib/systemd/system-preset/
mkdir -p /usr/lib/systemd/user-preset/

# Set Greetd e Podman come default (livello System)
echo "enable greetd.service" > /usr/lib/systemd/system-preset/99-ermeteos.preset
echo "enable podman.socket" >> /usr/lib/systemd/system-preset/99-ermeteos.preset

# Setup DMS service for all new users (livello User)
echo "enable dms.service" > /usr/lib/systemd/user-preset/99-ermeteos.preset

# Copy Niri dotfiles to skel
mkdir -p /etc/skel/.config/niri/
cp -rf /ctx/dot_config/niri/config.kdl /etc/skel/.config/niri/

# FIX: Assicura i permessi corretti per lo skeleton directory
chown -R root:root /etc/skel/
chmod -R 755 /etc/skel/

# Remove waybar
dnf -y remove waybar
