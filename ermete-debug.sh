#!/bin/bash
# ==============================================================================
# Ermete OS - Diagnostica Assoluta e Omnicomprensiva (v5.0 - X-RAY EDITION)
# La "Lastra a Raggi X" definitiva per debugging istantaneo.
# ==============================================================================

if [ "$EUID" -ne 0 ]; then
  echo "Per favore, esegui lo script come root (sudo ./ermete-debug.sh)"
  exit 1
fi

LOG_FILE="/tmp/ermete_diagnostic_$(date +%Y%m%d_%H%M%S).txt"
USER_ID=1000
USER_NAME="ermete"

echo "Avvio della diagnostica ESTREMA per Ermete OS (v5.0 X-RAY EDITION)..."
echo "Acquisizione radiale dell'Infrastruttura, GPU, Vulkan, DBus e User Space..."
echo "========================================" > "$LOG_FILE"
echo " ERMETE OS - OMNI-DIAGNOSTIC LOG v5.0 (X-RAY) " >> "$LOG_FILE"
echo " Timestamp: $(date)" >> "$LOG_FILE"
echo "========================================" >> "$LOG_FILE"

# --- 1. SYSTEM BASE, SELINUX & ZERO-TRUST REPOSITORIES ---
echo -e "\n\n========================================\n1. SYSTEM BASE & ZERO-TRUST\n========================================" >> "$LOG_FILE"
uname -a >> "$LOG_FILE" 2>&1
cat /etc/os-release >> "$LOG_FILE" 2>&1
echo -e "\n[Bootc / Ostree Status]" >> "$LOG_FILE"
bootc status >> "$LOG_FILE" 2>&1 || rpm-ostree status >> "$LOG_FILE" 2>&1
echo -e "\n[Kernel Cmdline]" >> "$LOG_FILE"
cat /proc/cmdline >> "$LOG_FILE" 2>&1
echo -e "\n[Secure Boot & MOK]" >> "$LOG_FILE"
mokutil --sb-state >> "$LOG_FILE" 2>&1
echo -e "\n[SELinux Status (sestatus)]" >> "$LOG_FILE"
sestatus >> "$LOG_FILE" 2>&1 || echo "SELinux non disponibile" >> "$LOG_FILE"
echo -e "\n[GPG Trust Anchors & Local Repositories]" >> "$LOG_FILE"
ls -la /etc/yum.repos.d/ /etc/pki/rpm-gpg/ >> "$LOG_FILE" 2>&1

# --- 2. PACKAGES, KERNEL & INITRAMFS ---
echo -e "\n\n========================================\n2. PACKAGES, KERNEL & DKMS\n========================================" >> "$LOG_FILE"
echo -e "\n[Installed Packages (Core Stack)]" >> "$LOG_FILE"
rpm -qa | grep -iE 'kernel-cachy|nvidia|dkms|niri|wayland|polkit|greetd|systemd|mesa|vulkan|egl' | sort >> "$LOG_FILE" 2>&1
echo -e "\n[DKMS Status]" >> "$LOG_FILE"
dkms status >> "$LOG_FILE" 2>&1
echo -e "\n[Initramfs / Dracut (NVIDIA & SYSUSERS)]" >> "$LOG_FILE"
lsinitrd | grep -iE 'nvidia|nouveau|group|passwd|sysusers|systemd-udev' >> "$LOG_FILE" 2>&1

# --- 3. HARDWARE, GPU, DRM, EGL & VULKAN (LASTRA X-RAY) ---
echo -e "\n\n========================================\n3. HARDWARE, GPU, DRM, EGL & VULKAN\n========================================" >> "$LOG_FILE"
echo -e "\n[Loaded Modules]" >> "$LOG_FILE"
lsmod | grep -iE 'nvidia|nouveau|video' >> "$LOG_FILE" 2>&1
echo -e "\n[NVIDIA SMI]" >> "$LOG_FILE"
nvidia-smi >> "$LOG_FILE" 2>&1
echo -e "\n[DRM & Modesetting parameters]" >> "$LOG_FILE"
ls -la /dev/dri >> "$LOG_FILE" 2>&1
echo "nvidia_drm modeset: $(cat /sys/module/nvidia_drm/parameters/modeset 2>/dev/null || echo 'N/A')" >> "$LOG_FILE"
echo "nvidia_drm fbdev: $(cat /sys/module/nvidia_drm/parameters/fbdev 2>/dev/null || echo 'N/A')" >> "$LOG_FILE"
echo -e "\n[Udevadm Info per card0 e renderD128]" >> "$LOG_FILE"
udevadm info -q all -n /dev/dri/card0 2>/dev/null >> "$LOG_FILE" 2>&1
udevadm info -q all -n /dev/dri/renderD128 2>/dev/null >> "$LOG_FILE" 2>&1

echo -e "\n[Vulkan ICD & EGL Hardware Acceleration]" >> "$LOG_FILE"
ls -la /usr/share/vulkan/icd.d/ /usr/share/glvnd/egl_vendor.d/ >> "$LOG_FILE" 2>&1
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID vulkaninfo --summary >> "$LOG_FILE" 2>&1 || echo "Vulkaninfo fallito o pacchetto vulkan-tools mancante" >> "$LOG_FILE"

echo -e "\n[Sensors & Throttling]" >> "$LOG_FILE"
sensors >> "$LOG_FILE" 2>&1 || echo "lm_sensors non disponibile" >> "$LOG_FILE"

# --- 4. IDENTITIES, GROUPS & IDEMPOTENCE ---
echo -e "\n\n========================================\n4. IDENTITIES & GROUPS\n========================================" >> "$LOG_FILE"
echo -e "\n[Contenuto /etc/group (Gruppi Vitali)]" >> "$LOG_FILE"
grep -iE 'video|render|input|tty|audio|disk|kvm|greetd' /etc/group >> "$LOG_FILE" 2>&1
echo -e "\n[Skel UNIX Permissions (Idempotenza & Privacy)]" >> "$LOG_FILE"
ls -la /etc/skel/ >> "$LOG_FILE" 2>&1
echo -e "\n[User Limits (ulimit -a)]" >> "$LOG_FILE"
sudo -u $USER_NAME bash -c "ulimit -a" >> "$LOG_FILE" 2>&1

# --- 5. SYSTEMD SERVICES & PAM ---
echo -e "\n\n========================================\n5. SYSTEMD SERVICES & PAM\n========================================" >> "$LOG_FILE"
echo -e "\n[FAILED SERVICES (System)]" >> "$LOG_FILE"
systemctl --failed >> "$LOG_FILE" 2>&1
echo -e "\n[Authselect / PAM Status]" >> "$LOG_FILE"
authselect current >> "$LOG_FILE" 2>&1
authselect check >> "$LOG_FILE" 2>&1
echo -e "\n[Display Manager Status]" >> "$LOG_FILE"
systemctl status greetd sddm gdm display-manager --no-pager >> "$LOG_FILE" 2>&1

# --- 6. LOGINCTL, POLKIT & DBUS ---
echo -e "\n\n========================================\n6. LOGINCTL, POLKIT & DBUS\n========================================" >> "$LOG_FILE"
loginctl list-sessions >> "$LOG_FILE" 2>&1
for session in $(loginctl list-sessions --no-legend | awk '{print $1}'); do
    echo "Session $session:" >> "$LOG_FILE"
    loginctl show-session "$session" >> "$LOG_FILE" 2>&1
done
echo -e "\n[Polkit Active Agents]" >> "$LOG_FILE"
ps aux | grep -iE 'polkit' >> "$LOG_FILE" 2>&1
echo -e "\n[DBus User Bus Introspection (Active Connections)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID busctl --user list --no-pager >> "$LOG_FILE" 2>&1 || echo "DBus non interrogabile" >> "$LOG_FILE"

# --- 7. NIRI, WAYLAND & NETWORKING ---
echo -e "\n\n========================================\n7. NIRI, ENV & NETWORKING\n========================================" >> "$LOG_FILE"
echo -e "\n[Greetd & Session Wrappers]" >> "$LOG_FILE"
cat /usr/bin/niri-session >> "$LOG_FILE" 2>/dev/null || echo "Wrapper non trovato" >> "$LOG_FILE"
echo -e "\n[Active Wayland/X11 Sockets]" >> "$LOG_FILE"
ls -la /run/user/*/wayland-* /tmp/.X11-unix/ >> "$LOG_FILE" 2>/dev/null || echo "No sockets found" >> "$LOG_FILE"
echo -e "\n[Networking & Firewalld (Network Agnostico)]" >> "$LOG_FILE"
echo "Firewalld State: $(firewall-cmd --state 2>/dev/null || echo 'Not running')" >> "$LOG_FILE"
echo "NM-wait-online: $(systemctl is-enabled NetworkManager-wait-online.service 2>/dev/null || echo 'Unknown')" >> "$LOG_FILE"

# --- 8. LATO USER: UX, FLATPAK, AUDIO & GSETTINGS ---
echo -e "\n\n========================================\n8. USER EXPERIENCE (UX), AUDIO & FLATPAK\n========================================" >> "$LOG_FILE"
echo -e "\n[User Systemd Services (Wayland Desktop Components)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID systemctl --user status niri-session.target ironbar.service swaybg.service pipewire.service wireplumber.service xdg-desktop-portal.service --no-pager >> "$LOG_FILE" 2>&1 || echo "Could not query user systemd services" >> "$LOG_FILE"
echo -e "\n[FAILED SERVICES (User)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID systemctl --user --failed >> "$LOG_FILE" 2>&1

echo -e "\n[Pipewire & Audio Status (wpctl)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID wpctl status >> "$LOG_FILE" 2>&1 || echo "Pipewire wpctl non disponibile" >> "$LOG_FILE"

echo -e "\n[Flatpak Configs & Overrides]" >> "$LOG_FILE"
flatpak list >> "$LOG_FILE" 2>&1 || echo "Nessun pacchetto Flatpak trovato" >> "$LOG_FILE"
cat /var/lib/flatpak/overrides/global >> "$LOG_FILE" 2>/dev/null || echo "Nessun global override" >> "$LOG_FILE"

echo -e "\n[GSettings / GTK Theming (Lato Utente)]" >> "$LOG_FILE"
sudo -u $USER_NAME dbus-launch gsettings get org.gnome.desktop.interface color-scheme >> "$LOG_FILE" 2>&1
sudo -u $USER_NAME dbus-launch gsettings get org.gnome.desktop.interface gtk-theme >> "$LOG_FILE" 2>&1
sudo -u $USER_NAME dbus-launch gsettings get org.gnome.desktop.interface icon-theme >> "$LOG_FILE" 2>&1

echo -e "\n[User Environment Variables (Wayland/DBus/Nvidia)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID sh -c 'env | grep -iE "wayland|nvidia|gbm|wlr|xdg|gtk"' >> "$LOG_FILE" 2>&1

echo -e "\n[Niri msg outputs (Geometria Monitor)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID niri msg outputs >> "$LOG_FILE" 2>&1 || echo "Niri non accessibile o disattivo" >> "$LOG_FILE"

# --- 9. SECURITY LOGS & CRASH DUMPS (SELINUX INCLUDED) ---
echo -e "\n\n========================================\n9. FULL CRITICAL LOGS, SELINUX & COREDUMPS\n========================================" >> "$LOG_FILE"
echo -e "\n[SELinux AVC Denials]" >> "$LOG_FILE"
ausearch -m AVC,USER_AVC,SELINUX_ERR,MAC_POLICY_LOAD -ts boot >> "$LOG_FILE" 2>&1 || echo "Nessun AVC Denial trovato o ausearch non installato" >> "$LOG_FILE"

echo -e "\n[DMESG - Graphic/Boot Errors]" >> "$LOG_FILE"
dmesg | grep -iE 'nvidia|nouveau|secure boot|lockdown|error|fail|drm|wayland|niri|udev|segfault' >> "$LOG_FILE" 2>&1

echo -e "\n[JOURNALCTL - All Wayland/Niri/Portals/DBUS/Greetd Traces]" >> "$LOG_FILE"
journalctl -b | grep -iE 'niri|wayland|wlroots|pipewire|wireplumber|dbus|polkit|xdg-desktop-portal|greetd|login|ironbar|swaybg' | tail -n 1000 >> "$LOG_FILE" 2>&1

echo -e "\n[JOURNALCTL - User Errors (Priority 3)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID journalctl --user -b -p 3 --no-pager >> "$LOG_FILE" 2>&1 || echo "Nessun log d'errore utente." >> "$LOG_FILE"

echo -e "\n[COREDUMPCTL - Recent Crashes]" >> "$LOG_FILE"
coredumpctl list --no-legend | tail -n 20 >> "$LOG_FILE" 2>&1

echo -e "\n========================================" >> "$LOG_FILE"
echo " OMNI-DIAGNOSTIC COMPLETE" >> "$LOG_FILE"
echo "========================================" >> "$LOG_FILE"

echo "Scansione X-RAY completata. Log inviato."
echo "------------------------------------------------------"

if ping -q -c 1 -W 1 8.8.8.8 >/dev/null 2>&1; then
    UPLOAD_URL=$(curl -s --data-binary @"$LOG_FILE" https://paste.rs/)
    if [[ $UPLOAD_URL == http* ]]; then
        echo -e "\n✅ SUCCESSO ASSOLUTO! Il super-log è online."
        echo "🔗 COPIA E INVIA QUESTO URL AL BOT: $UPLOAD_URL"
    else
        echo "❌ Upload fallito (forse log troppo grande per paste.rs)."
    fi
else
    echo "Nessuna connessione di rete. Copia $LOG_FILE manualmente."
fi
