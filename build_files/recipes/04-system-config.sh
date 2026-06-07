#!/bin/bash
set -ouex pipefail

echo "--- Configuring system services and user defaults ---"

mkdir -p /etc/greetd/

# FIX: Creazione di un profilo Niri in confinamento isolato per prevenire Privilege Escalation (LPE)
# via Mod+T o manipolazioni non autorizzate sulla lock screen.
cat > /etc/greetd/niri.kdl << 'EOF'
hotkey-overlay {
    skip-at-startup
}
environment {
    DMS_RUN_GREETER "1"
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

# Install greetd login manager vincolato alla configurazione restrittiva appena creata
cat > /etc/greetd/config.toml << EOF
[terminal]
vt = 1
[default_session]
user = "greeter"
command = "dms-greeter --command niri -C /etc/greetd/niri.kdl"
EOF

# Architettura Systemd nativa (Systemd Presets)
mkdir -p /usr/lib/systemd/system-preset/
mkdir -p /usr/lib/systemd/user-preset/

# Set Greetd e Podman come default attivi (livello System)
echo "enable greetd.service" > /usr/lib/systemd/system-preset/99-ermeteos.preset
echo "enable podman.socket" >> /usr/lib/systemd/system-preset/99-ermeteos.preset

# Setup DMS service per l'inizializzazione dell'ambiente Wayland (livello User)
echo "enable dms.service" > /usr/lib/systemd/user-preset/99-ermeteos.preset

# Copy Niri dotfiles to skel
mkdir -p /etc/skel/.config/niri/
cp -rf /ctx/dot_config/niri/config.kdl /etc/skel/.config/niri/

# FIX: Assicura i permessi corretti per lo skeleton directory garantendo la Privacy
# dei futuri utenti. Le directory diventano non-attraversabili, i file non-eseguibili.
chown -R root:root /etc/skel/
find /etc/skel/ -type d -exec chmod 700 {} \;
find /etc/skel/ -type f -exec chmod 600 {} \;

# Remove waybar
dnf -y remove waybar
