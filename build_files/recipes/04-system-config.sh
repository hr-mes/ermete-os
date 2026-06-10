#!/bin/bash
set -ouex pipefail

echo "--- Configuring system services and user defaults ---"

mkdir -p /etc/greetd/

# Creazione di un profilo Niri in confinamento isolato per prevenire Privilege Escalation (LPE)
cat > /etc/greetd/niri.kdl << 'EOF'
hotkey-overlay {
    skip-at-startup
}
layout {
    background-color "#000000"
}
binds {
    // Esclusivamente controlli hardware e spegnimento.
    // L'ESECUZIONE DI TERMINALI O LANCIATORI APPLICATIVI È SEVERAMENTE INIBITA.
    XF86AudioRaiseVolume allow-when-locked=true { spawn "wpctl" "set-volume" "@DEFAULT_AUDIO_SINK@" "0.1+" "-l" "1.0"; }
    XF86AudioLowerVolume allow-when-locked=true { spawn "wpctl" "set-volume" "@DEFAULT_AUDIO_SINK@" "0.1-"; }
    XF86AudioMute        allow-when-locked=true { spawn "wpctl" "set-mute" "@DEFAULT_AUDIO_SINK@" "toggle"; }
    Mod+Shift+E { quit; }
    Ctrl+Alt+Delete { quit; }
}
EOF

# Install greetd login manager vincolato alla configurazione restrittiva con tuigreet
cat > /etc/greetd/config.toml << EOF
[terminal]
vt = 1
[default_session]
user = "greeter"
command = "tuigreet --time --cmd niri"
EOF

# Architettura Systemd nativa (Systemd Presets)
mkdir -p /usr/lib/systemd/system-preset/
mkdir -p /usr/lib/systemd/user-preset/

# Set Greetd e Podman come default attivi (livello System)
echo "enable greetd.service" > /usr/lib/systemd/system-preset/99-ermeteos.preset
echo "enable podman.socket" >> /usr/lib/systemd/system-preset/99-ermeteos.preset
echo "disable NetworkManager-wait-online.service" >> /usr/lib/systemd/system-preset/99-ermeteos.preset
echo "enable firewalld.service" >> /usr/lib/systemd/system-preset/99-ermeteos.preset

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

# Copy Niri dotfiles to skel
mkdir -p /etc/skel/.config/niri/
cp -rf /ctx/dot_config/niri/config.kdl /etc/skel/.config/niri/

# Abilita Starship (Prompt in Rust) globalmente per le shell compatibili
echo 'eval "$(starship init bash)"' > /etc/profile.d/starship.sh

# Assicura i permessi corretti per lo skeleton directory garantendo la Privacy
chown -R root:root /etc/skel/
find /etc/skel/ -type d -exec chmod 700 {} \;
find /etc/skel/ -type f -exec chmod 600 {} \;

# Remove waybar
dnf -y remove waybar
