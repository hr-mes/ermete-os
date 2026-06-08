#!/bin/bash
set -ouex pipefail

echo "--- Configuring system services and user defaults ---"

mkdir -p /etc/greetd/

# 1. FIX LPE: Creazione di una configurazione Niri minimale ed ermetica per Greetd.
# Inibisce qualsiasi input o escape manuale e avvia ReGreet in confinamento.
cat > /etc/greetd/niri-regreet.kdl << 'EOF'
hotkey-overlay {
    skip-at-startup
}
window-rule {
    geometry-corner-radius 0
    clip-to-geometry true
}
// Avvia unicamente il greeter in Rust
spawn-at-startup "regreet"
// Nessuna scorciatoia da tastiera configurata pre-autenticazione
binds {}
EOF

# Configurazione di ReGreet (login manager)
cat > /etc/greetd/regreet.toml << 'EOF'
[background]
path = "/usr/share/backgrounds/default.png"
fit = "Cover"


application_prefer_dark_theme = true
cursor_theme_name = "Adwaita"
font_name = "Cantarell 11"
icon_theme_name = "Adwaita"
theme_name = "Adwaita"
EOF

# Configurazione principale di greetd
cat > /etc/greetd/config.toml << EOF
[terminal]
vt = 1
[default_session]
user = "greeter"
command = "niri --config /etc/greetd/niri-regreet.kdl"
EOF

# 2. Definizione servizi Systemd utente per l'ambiente Rust
mkdir -p /usr/lib/systemd/user/
mkdir -p /usr/lib/systemd/system-preset/
mkdir -p /usr/lib/systemd/user-preset/

# Servizio utente per la barra Ironbar
cat > /usr/lib/systemd/user/ironbar.service << 'EOF'
[Unit]
Description=Ironbar GTK4 Status Bar (Rust)
PartOf=graphical-session.target
After=graphical-session-pre.target


ExecStart=/usr/bin/ironbar
Restart=on-failure

[Install]
WantedBy=graphical-session.target
EOF

# Servizio utente per il demone degli sfondi Swww
cat > /usr/lib/systemd/user/swww.service << 'EOF'
[Unit]
Description=Swww Wayland Wallpaper Daemon (Rust)
PartOf=graphical-session.target
After=graphical-session-pre.target


ExecStart=/usr/bin/swww-daemon
ExecStartPost=/usr/bin/swww img /usr/share/backgrounds/default.png
Restart=on-failure

[Install]
WantedBy=graphical-session.target
EOF

# Configurazione Presets (abilitazione automatica dei servizi)
echo "enable greetd.service" > /usr/lib/systemd/system-preset/99-ermeteos.preset
echo "enable podman.socket" >> /usr/lib/systemd/system-preset/99-ermeteos.preset

echo "enable ironbar.service" > /usr/lib/systemd/user-preset/99-ermeteos.preset
echo "enable swww.service" >> /usr/lib/systemd/user-preset/99-ermeteos.preset

# 3. Preparazione directory di default per i nuovi utenti
mkdir -p /etc/skel/.config/niri/
mkdir -p /etc/skel/.config/ironbar/
mkdir -p /etc/skel/.config/anyrun/

# Copia dei file di configurazione degli strumenti Rust nello skel
cp -rf /ctx/dot_config/niri/config.kdl /etc/skel/.config/niri/
cp -rf /ctx/dot_config/ironbar/config.json /etc/skel/.config/ironbar/
cp -rf /ctx/dot_config/anyrun/config.ron /etc/skel/.config/anyrun/

# Blindatura dei permessi dello skeleton per il principio del privilegio minimo
chown -R root:root /etc/skel/
find /etc/skel/ -type d -exec chmod 700 {} \;
find /etc/skel/ -type f -exec chmod 600 {} \;

# Rimozione pacchetti non necessari
dnf -y remove waybar
