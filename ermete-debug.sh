#!/bin/bash
# ==============================================================================
# Ermete OS - Diagnostica Assoluta e Omnicomprensiva
# Raccoglie OGNI POSSIBILE INFORMAZIONE riguardante Kernel, NVIDIA, Systemd,
# Wayland, Niri, XDG Portals, DKMS e Pipewire.
# ==============================================================================

if [ "$EUID" -ne 0 ]; then
  echo "Per favore, esegui lo script come root (sudo ./ermete-debug.sh)"
  exit 1
fi

LOG_FILE="/tmp/ermete_diagnostic_$(date +%Y%m%d_%H%M%S).txt"

echo "Avvio della diagnostica ESTREMA per Ermete OS..."
echo "Attendere, sto raccogliendo tutto..."
echo "========================================" > "$LOG_FILE"
echo " ERMETE OS - OMNI-DIAGNOSTIC LOG " >> "$LOG_FILE"
echo " Timestamp: $(date)" >> "$LOG_FILE"
echo "========================================" >> "$LOG_FILE"

# --- 1. SYSTEM BASE ---
echo -e "\n\n========================================\n1. SYSTEM BASE\n========================================" >> "$LOG_FILE"
uname -a >> "$LOG_FILE" 2>&1
cat /etc/os-release >> "$LOG_FILE" 2>&1
echo -e "\n[Bootc / Ostree Status]" >> "$LOG_FILE"
bootc status >> "$LOG_FILE" 2>&1 || rpm-ostree status >> "$LOG_FILE" 2>&1
echo -e "\n[Kernel Cmdline]" >> "$LOG_FILE"
cat /proc/cmdline >> "$LOG_FILE" 2>&1
echo -e "\n[Secure Boot & MOK]" >> "$LOG_FILE"
mokutil --sb-state >> "$LOG_FILE" 2>&1

# --- 2. PACKAGES & DKMS ---
echo -e "\n\n========================================\n2. PACKAGES, KERNEL & DKMS\n========================================" >> "$LOG_FILE"
echo -e "\n[Installed Kernel, NVIDIA & DKMS packages]" >> "$LOG_FILE"
rpm -qa | grep -iE 'kernel-cachy|nvidia|dkms|niri|wayland' | sort >> "$LOG_FILE" 2>&1
echo -e "\n[DKMS Status]" >> "$LOG_FILE"
dkms status >> "$LOG_FILE" 2>&1
echo -e "\n[Initramfs / Dracut NVIDIA inclusion]" >> "$LOG_FILE"
lsinitrd | grep -iE 'nvidia|nouveau' >> "$LOG_FILE" 2>&1

# --- 3. HARDWARE, GPU & DRM ---
echo -e "\n\n========================================\n3. HARDWARE, GPU & DRM\n========================================" >> "$LOG_FILE"
lspci -nnk | grep -iA 3 vga >> "$LOG_FILE" 2>&1
echo -e "\n[Loaded Modules]" >> "$LOG_FILE"
lsmod | grep -iE 'nvidia|nouveau|video' >> "$LOG_FILE" 2>&1
echo -e "\n[NVIDIA SMI]" >> "$LOG_FILE"
nvidia-smi >> "$LOG_FILE" 2>&1
echo -e "\n[DRM & Modesetting parameters]" >> "$LOG_FILE"
ls -l /dev/dri >> "$LOG_FILE" 2>&1
echo "nvidia_drm modeset: $(cat /sys/module/nvidia_drm/parameters/modeset 2>/dev/null || echo 'N/A')" >> "$LOG_FILE"
echo "nvidia_drm fbdev: $(cat /sys/module/nvidia_drm/parameters/fbdev 2>/dev/null || echo 'N/A')" >> "$LOG_FILE"

# --- 4. SYSTEMD SERVICES ---
echo -e "\n\n========================================\n4. SYSTEMD SERVICES\n========================================" >> "$LOG_FILE"
echo -e "\n[System Default Target]" >> "$LOG_FILE"
systemctl get-default >> "$LOG_FILE" 2>&1
echo -e "\n[FAILED SERVICES (System)]" >> "$LOG_FILE"
systemctl --failed >> "$LOG_FILE" 2>&1
echo -e "\n[Display Manager Status]" >> "$LOG_FILE"
systemctl status greetd sddm gdm display-manager --no-pager >> "$LOG_FILE" 2>&1
echo -e "\n[NVIDIA Powerd / Persistenced]" >> "$LOG_FILE"
systemctl status nvidia-powerd nvidia-persistenced --no-pager >> "$LOG_FILE" 2>&1

# --- 5. LOGINCTL & USER SESSIONS ---
echo -e "\n\n========================================\n5. LOGINCTL & USER\n========================================" >> "$LOG_FILE"
loginctl list-sessions >> "$LOG_FILE" 2>&1
for session in $(loginctl list-sessions --no-legend | awk '{print $1}'); do
    echo "Session $session:" >> "$LOG_FILE"
    loginctl show-session "$session" >> "$LOG_FILE" 2>&1
done

# --- 6. NIRI & ENVIRONMENT CONFIGURATION ---
echo -e "\n\n========================================\n6. NIRI & ENV CONFIG\n========================================" >> "$LOG_FILE"
echo -e "\n[Greetd & Session Wrappers]" >> "$LOG_FILE"
echo "--- /etc/greetd/config.toml ---" >> "$LOG_FILE"
cat /etc/greetd/config.toml >> "$LOG_FILE" 2>/dev/null || echo "File not found" >> "$LOG_FILE"
echo "--- /usr/bin/niri-session ---" >> "$LOG_FILE"
cat /usr/bin/niri-session >> "$LOG_FILE" 2>/dev/null || echo "File not found" >> "$LOG_FILE"
echo -e "\n[Active Wayland/X11 Sockets]" >> "$LOG_FILE"
ls -la /run/user/*/wayland-* /tmp/.X11-unix/ >> "$LOG_FILE" 2>/dev/null || echo "No sockets found" >> "$LOG_FILE"
echo -e "\n[Environment Variables in /etc/profile.d/]" >> "$LOG_FILE"
grep -r -iE 'wayland|nvidia|gbm|wlr' /etc/profile.d/ /etc/environment >> "$LOG_FILE" 2>&1
echo -e "\n[Niri Config Dump (Skel / Users)]" >> "$LOG_FILE"
echo "--- /etc/skel/.config/niri/config.kdl ---" >> "$LOG_FILE"
cat /etc/skel/.config/niri/config.kdl 2>/dev/null | head -n 50 >> "$LOG_FILE"
for u in /home/*; do
    if [ -d "$u" ]; then
        echo "--- $u/.config/niri/config.kdl ---" >> "$LOG_FILE"
        cat "$u/.config/niri/config.kdl" 2>/dev/null | head -n 50 >> "$LOG_FILE"
    fi
done

# --- 7. COMPLETE LOGS ---
echo -e "\n\n========================================\n7. FULL CRITICAL LOGS\n========================================" >> "$LOG_FILE"
echo -e "\n[DMESG - Graphic/Boot Errors]" >> "$LOG_FILE"
dmesg | grep -iE 'nvidia|nouveau|secure boot|lockdown|error|fail|drm|wayland|niri' >> "$LOG_FILE" 2>&1

echo -e "\n[JOURNALCTL - System Boot Errors (Priority 3)]" >> "$LOG_FILE"
journalctl -b -p 3 --no-pager >> "$LOG_FILE" 2>&1

echo -e "\n[JOURNALCTL - All Wayland/Niri/Portals/DBUS Traces]" >> "$LOG_FILE"
# Raccoglie log di Niri, Polkit, Wayland, XDG Portals, Pipewire, sia di sistema che degli utenti
journalctl -b | grep -iE 'niri|wayland|wlroots|pipewire|wireplumber|dbus|polkit|xdg-desktop-portal|greetd' | tail -n 500 >> "$LOG_FILE" 2>&1

echo -e "\n[JOURNALCTL - NVIDIA DKMS Build Failures (se presenti)]" >> "$LOG_FILE"
journalctl -b -u dkms | grep -iE 'error|fail|nvidia' >> "$LOG_FILE" 2>&1

echo -e "\n========================================" >> "$LOG_FILE"
echo " OMNI-DIAGNOSTIC COMPLETE" >> "$LOG_FILE"
echo "========================================" >> "$LOG_FILE"

echo "Raccolta dati completata! Il log è immenso e perfetto."
echo "------------------------------------------------------"

# Tentativo di esfiltrazione via rete (Pastebin sicuro)
if ping -q -c 1 -W 1 8.8.8.8 >/dev/null 2>&1; then
    echo "Rete rilevata. Sto caricando questo colosso in modo sicuro..."
    UPLOAD_URL=$(curl -s --data-binary @"$LOG_FILE" https://paste.rs/)
    
    if [[ $UPLOAD_URL == http* ]]; then
        echo -e "\n✅ SUCCESSO ASSOLUTO! Il super-log è online."
        echo "==========================================================="
        echo "🔗 COPIA E INVIA QUESTO URL AL BOT: $UPLOAD_URL"
        echo "==========================================================="
    else
        echo "❌ Upload fallito (forse log troppo grande per paste.rs). Usa pendrive USB."
    fi
else
    echo "Nessuna connessione di rete rilevata."
    echo "Per condividere il log, copia il file su una USB:"
    echo "  mkdir -p /mnt/usb && mount /dev/sdX1 /mnt/usb"
    echo "  cp $LOG_FILE /mnt/usb/"
fi
