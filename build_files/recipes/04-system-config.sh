#!/bin/bash
set -ouex pipefail

echo "--- Configuring system services and user defaults ---"

mkdir -p /etc/greetd/


# Install greetd login manager vincolato alla configurazione restrittiva con tuigreet
cat > /etc/greetd/config.toml << EOF
[terminal]
vt = 1
[default_session]
user = "greeter"
command = "tuigreet --time --greeting 'Welcome to Ermete OS - Atomic Wayland' --asterisks --cmd niri"
EOF

# Architettura Systemd nativa (Systemd Presets)
mkdir -p /usr/lib/systemd/system-preset/
mkdir -p /usr/lib/systemd/user-preset/

# Aggressive OOMD configuration to protect Wayland session
mkdir -p /etc/systemd/oomd.conf.d/
cat > /etc/systemd/oomd.conf.d/10-ermete.conf << 'EOF'
[OOM]
DefaultMemoryPressureLimit=90%
DefaultMemoryPressureDurationSec=5
EOF

# Set Greetd, Podman, and OOMD come default attivi (livello System)
echo "enable greetd.service" > /usr/lib/systemd/system-preset/99-Ermete.preset
echo "enable podman.socket" >> /usr/lib/systemd/system-preset/99-Ermete.preset
echo "enable nix-daemon.socket" >> /usr/lib/systemd/system-preset/99-Ermete.preset
echo "enable systemd-oomd.service" >> /usr/lib/systemd/system-preset/99-Ermete.preset
echo "enable bootc-fetch-apply.timer" >> /usr/lib/systemd/system-preset/99-Ermete.preset
echo "disable NetworkManager-wait-online.service" >> /usr/lib/systemd/system-preset/99-Ermete.preset
echo "enable firewalld.service" >> /usr/lib/systemd/system-preset/99-Ermete.preset

# 2. Firewalld Zero-Trust (Blocca tutto il traffico in ingresso per default)
echo "Configuring Firewalld default zone to drop..."
mkdir -p /etc/firewalld/
cat > /etc/firewalld/firewalld.conf << 'EOF'
DefaultZone=drop
EOF

# Disabilita i Coredump su disco per privacy totale
mkdir -p /etc/systemd/coredump.conf.d/
cat > /etc/systemd/coredump.conf.d/disable.conf << EOF
[Coredump]
Storage=none
EOF

# Randomizzazione MAC Address Wi-Fi/Ethernet (Anti-Tracking Fisico)
mkdir -p /etc/NetworkManager/conf.d/
cat > /etc/NetworkManager/conf.d/00-macrandomize.conf << EOF
[device]
wifi.scan-rand-mac-address=yes

[connection]
wifi.cloned-mac-address=random
ethernet.cloned-mac-address=random
EOF

# DNS-over-TLS (DoT) Forzato (Anti-Tracciamento ISP)
mkdir -p /etc/systemd/resolved.conf.d/
cat > /etc/systemd/resolved.conf.d/dns_over_tls.conf << EOF
[Resolve]
DNSOverTLS=yes
EOF

# Copy all dotfiles to skel
mkdir -p /etc/skel/.config/
cp -rf /ctx/dot_config/* /etc/skel/.config/

# Abilita Starship (Prompt in Rust) globalmente per le shell compatibili
echo 'eval "$(starship init bash)"' > /etc/profile.d/starship.sh

# Assicura i permessi corretti per lo skeleton directory garantendo la Privacy
# senza rompere gli script (forzando +x sui file .sh)
chown -R root:root /etc/skel/
find /etc/skel -type d -exec chmod 700 {} \;
find /etc/skel -type f -exec chmod 600 {} \;
find /etc/skel -type f -name "*.sh" -exec chmod 700 {} \;

# Remove waybar
dnf -y remove waybar
