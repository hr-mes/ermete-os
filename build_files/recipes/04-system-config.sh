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

# OOMD delegato ai default di sistema bilanciati (evita il collasso del cgroup Niri/Wayland)
# Rimosso il tuning aggressivo che causava Denial of Service della sessione grafica.

# Set Greetd, Podman, and OOMD come default attivi (livello System)
echo "enable greetd.service" > /usr/lib/systemd/system-preset/99-Ermete.preset
echo "enable podman.socket" >> /usr/lib/systemd/system-preset/99-Ermete.preset
echo "enable nix-daemon.socket" >> /usr/lib/systemd/system-preset/99-Ermete.preset

# Provisioning Dinamico per Nix
# Il pacchetto Nix è stato spacchettato in /usr/share/nix-base durante la build per aggirare
# il limite del rootfs immutabile. Systemd-tmpfiles copierà i file in /var/nix (se vuoto) al boot.
mkdir -p /usr/lib/tmpfiles.d/
cat > /usr/lib/tmpfiles.d/nix-provisioning.conf << 'EOF'
d /var/nix 0755 root root - -
C /var/nix - - - - /usr/share/nix-base
EOF
echo "enable systemd-oomd.service" >> /usr/lib/systemd/system-preset/99-Ermete.preset
echo "enable bootc-fetch-apply.timer" >> /usr/lib/systemd/system-preset/99-Ermete.preset
echo "disable NetworkManager-wait-online.service" >> /usr/lib/systemd/system-preset/99-Ermete.preset
echo "enable firewalld.service" >> /usr/lib/systemd/system-preset/99-Ermete.preset

# 2. Firewalld Zero-Trust (Blocca traffico in ingresso, ma preserva la UX locale)
echo "Configuring Firewalld default zone to drop..."
mkdir -p /etc/firewalld/
cat > /etc/firewalld/firewalld.conf << 'EOF'
DefaultZone=drop
EOF

# Preserva mDNS nella drop zone per la scoperta vitale di stampanti, scanner e Chromecast
mkdir -p /etc/firewalld/zones/
cat > /etc/firewalld/zones/drop.xml << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<zone target="DROP">
  <short>Drop</short>
  <description>Unsolicited incoming network packets are dropped. Incoming packets that are related to outgoing network connections are accepted.</description>
  <service name="mdns"/>
</zone>
EOF

# Disabilita i Coredump su disco per privacy totale
mkdir -p /etc/systemd/coredump.conf.d/
cat > /etc/systemd/coredump.conf.d/disable.conf << EOF
[Coredump]
Storage=none
EOF

# Randomizzazione MAC Address Wi-Fi/Ethernet (Privacy senza rompere DHCP e Portali Captive)
# L'uso di "stable" genera un MAC crittografato fittizio ma costante per ogni specifica rete
mkdir -p /etc/NetworkManager/conf.d/
cat > /etc/NetworkManager/conf.d/00-macrandomize.conf << EOF
[device]
wifi.scan-rand-mac-address=yes

[connection]
wifi.cloned-mac-address=stable
ethernet.cloned-mac-address=stable
EOF

# DNS-over-TLS (DoT) Opportunistico (Anti-Tracciamento agnostico)
# Se il DNS remoto supporta DoT lo usa, altrimenti fa fallback in chiaro per evitare Denial of Service
mkdir -p /etc/systemd/resolved.conf.d/
cat > /etc/systemd/resolved.conf.d/dns_over_tls.conf << EOF
[Resolve]
DNSOverTLS=opportunistic
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
