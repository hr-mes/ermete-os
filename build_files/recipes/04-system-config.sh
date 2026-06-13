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
command = "tuigreet --time --greeting 'Welcome to Ermete OS - Atomic Wayland' --asterisks --cmd /usr/bin/niri-session"
EOF

# Architettura Systemd nativa (Systemd Presets)
mkdir -p /usr/lib/systemd/system-preset/
mkdir -p /usr/lib/systemd/user-preset/

# OOMD delegato ai default di sistema bilanciati (evita il collasso del cgroup Niri/Wayland)
# Rimosso il tuning aggressivo che causava Denial of Service della sessione grafica.

# Set Greetd, Podman, and OOMD come default attivi (livello System)
echo "enable greetd.service" > /usr/lib/systemd/system-preset/99-Ermete.preset
echo "enable podman.socket" >> /usr/lib/systemd/system-preset/99-Ermete.preset

# Script helper per l'installazione di Nix a runtime (Zero-Trust su OSTree)
cat > /usr/bin/install-nix << 'EOF'
#!/bin/bash
echo "--- Installazione di Nix Package Manager (Determinate Systems) ---"
echo "Questo script installerà Nix in modo compatibile e sicuro con il file system immutabile."
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
EOF
chmod +x /usr/bin/install-nix
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

# Crea la directory degli screenshot per i nuovi utenti
mkdir -p /etc/skel/Pictures/Screenshots

# Abilita Starship (Prompt in Rust) globalmente per le shell compatibili
echo 'eval "$(starship init bash)"' > /etc/profile.d/starship.sh

# Assicura i permessi corretti per lo skeleton directory garantendo la Privacy
# senza rompere gli script (forzando +x sui file .sh)
chown -R root:root /etc/skel/
find /etc/skel -type d -exec chmod 700 {} \;
find /etc/skel -type f -exec chmod 600 {} \;
find /etc/skel -type f -name "*.sh" -exec chmod 700 {} \;

# 3. Btrfs Auto-Snapshot for /var/home (Zero-Maintenance Rollback)
# Crea lo script idempotente per lo snapshot
cat > /usr/libexec/ermete-snapshot.sh << 'EOF'
#!/bin/bash
set -euo pipefail

# Controlla se /var/home esiste ed è un subvolume valido
if [ ! -d "/var/home" ]; then
    exit 0
fi

mkdir -p /var/home/.snapshots

TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
# Se fallisce (es. rootfs non btrfs), esce senza bloccare l'aggiornamento
btrfs subvolume snapshot /var/home "/var/home/.snapshots/home_$TIMESTAMP" || exit 0

# Rotazione: mantieni solo gli ultimi 3 snapshot (salta i primi 3 più recenti, elimina gli altri)
cd /var/home/.snapshots
ls -1d home_* 2>/dev/null | sort -r | tail -n +4 | xargs -r btrfs subvolume delete || true
EOF
chmod +x /usr/libexec/ermete-snapshot.sh

# Crea il servizio systemd vincolato a ostree-finalize-staged
cat > /etc/systemd/system/ermete-home-snapshot.service << 'EOF'
[Unit]
Description=Ermete OS - Auto Btrfs Snapshot for /var/home
Before=ostree-finalize-staged.service
ConditionPathExists=/var/home

[Service]
Type=oneshot
ExecStart=/usr/libexec/ermete-snapshot.sh

[Install]
WantedBy=ostree-finalize-staged.service
EOF

echo "enable ermete-home-snapshot.service" >> /usr/lib/systemd/system-preset/99-Ermete.preset

# Remove waybar
dnf -y remove waybar
