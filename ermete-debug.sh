#!/bin/bash
# ==============================================================================
# Ermete OS - Diagnostica Assoluta e Omnicomprensiva (v8.0 - OMNI-VISION SUPREME)
# Visione Completa Verticale (Hardware -> App) e Orizzontale (IPC, Cgroups, Rete).
# ==============================================================================

if [ "$EUID" -ne 0 ]; then
  echo "Per favore, esegui lo script come root (sudo ./ermete-debug.sh)"
  exit 1
fi

LOG_FILE="/tmp/ermete_diagnostic_$(date +%Y%m%d_%H%M%S).txt"
USER_ID=1000
USER_NAME="ermete"
SESSION_ID=$(loginctl list-sessions | grep $USER_NAME | awk '{print $1}' | head -n 1)

echo "Avvio della diagnostica SUPREMA per Ermete OS (v8.0 OMNI-VISION SUPREME)..."
echo "Acquisizione dello Spettro Orizzontale, Verticale e Strutturale..."
echo "========================================" > "$LOG_FILE"
echo " ERMETE OS - OMNI-DIAGNOSTIC LOG v8.0 (OMNI-VISION SUPREME) " >> "$LOG_FILE"
echo " Timestamp: $(date)" >> "$LOG_FILE"
echo "========================================" >> "$LOG_FILE"

# --- 1. VISIONE VERTICALE: HARDWARE, EFI, ACPI & STORAGE ---
echo -e "\n\n========================================\n1. VERTICAL: HARDWARE, EFI, CPU & STORAGE\n========================================" >> "$LOG_FILE"
uname -a >> "$LOG_FILE" 2>&1
echo -e "\n[CPU, Microcode & Topology]" >> "$LOG_FILE"
lscpu | grep -iE 'Model name|Architecture|Vulnerability|Microcode|Thread' >> "$LOG_FILE" 2>&1
echo -e "\n[Firmware Updates (fwupd)]" >> "$LOG_FILE"
fwupdmgr get-devices --show-all-devices >> "$LOG_FILE" 2>&1 || echo "fwupdmgr non disponibile" >> "$LOG_FILE"
echo -e "\n[PCI Devices & Kernel Drivers (lspci -nnk)]" >> "$LOG_FILE"
lspci -nnk >> "$LOG_FILE" 2>&1
echo -e "\n[USB Topology (lsusb -t)]" >> "$LOG_FILE"
lsusb -t >> "$LOG_FILE" 2>&1
echo -e "\n[EFI Boot Manager & Variables]" >> "$LOG_FILE"
efibootmgr -v >> "$LOG_FILE" 2>&1 || echo "efibootmgr non disponibile" >> "$LOG_FILE"
echo -e "\n[Block Devices & Filesystems]" >> "$LOG_FILE"
lsblk -o NAME,FSTYPE,FSVER,LABEL,MOUNTPOINT,SIZE,RO,TYPE,UUID >> "$LOG_FILE" 2>&1
echo -e "\n[BTRFS Diagnostics & Subvolumes]" >> "$LOG_FILE"
btrfs filesystem show / >> "$LOG_FILE" 2>&1 || echo "Not BTRFS" >> "$LOG_FILE"
btrfs subvolume list / >> "$LOG_FILE" 2>&1 || echo "Not BTRFS" >> "$LOG_FILE"

# --- 2. VISIONE VERTICALE: OCI BOOTABLE, ZERO-TRUST & REPOSITORIES ---
echo -e "\n\n========================================\n2. VERTICAL: OCI BOOTABLE & REPOSITORIES\n========================================" >> "$LOG_FILE"
cat /etc/os-release >> "$LOG_FILE" 2>&1
echo -e "\n[Bootc / Ostree Status]" >> "$LOG_FILE"
bootc status >> "$LOG_FILE" 2>&1 || rpm-ostree status >> "$LOG_FILE" 2>&1
echo -e "\n[Secure Boot, MOK & Lockdown]" >> "$LOG_FILE"
mokutil --sb-state >> "$LOG_FILE" 2>&1
cat /sys/kernel/security/lockdown 2>/dev/null >> "$LOG_FILE" 2>&1
echo -e "\n[GPG Trust Anchors & Local Repositories]" >> "$LOG_FILE"
ls -la /etc/yum.repos.d/ /etc/pki/rpm-gpg/ >> "$LOG_FILE" 2>&1

# --- 3. VISIONE VERTICALE: KERNEL, DKMS, DRM, EGL & VULKAN ---
echo -e "\n\n========================================\n3. VERTICAL: KERNEL, DKMS, DRM & GPU\n========================================" >> "$LOG_FILE"
echo -e "\n[Kernel Cmdline & Boot Time]" >> "$LOG_FILE"
cat /proc/cmdline >> "$LOG_FILE" 2>&1
systemd-analyze time >> "$LOG_FILE" 2>&1
echo -e "\n[Installed Packages (Core Stack)]" >> "$LOG_FILE"
rpm -qa | grep -iE 'kernel-cachy|nvidia|dkms|mesa|vulkan|egl|wayland|niri' | sort >> "$LOG_FILE" 2>&1
echo -e "\n[DKMS & Module Status]" >> "$LOG_FILE"
dkms status >> "$LOG_FILE" 2>&1
lsmod | grep -iE 'nvidia|nouveau|video|drm' >> "$LOG_FILE" 2>&1
echo -e "\n[Initramfs / Dracut (NVIDIA & SYSUSERS)]" >> "$LOG_FILE"
lsinitrd | grep -iE 'nvidia|nouveau|group|passwd|sysusers|systemd-udev' >> "$LOG_FILE" 2>&1
echo -e "\n[NVIDIA SMI & DRM Parameters]" >> "$LOG_FILE"
nvidia-smi >> "$LOG_FILE" 2>&1 || echo "nvidia-smi non trovato" >> "$LOG_FILE"
cat /proc/driver/nvidia/version 2>/dev/null >> "$LOG_FILE" 2>&1
ls -la /dev/dri >> "$LOG_FILE" 2>&1
echo "nvidia_drm modeset: $(cat /sys/module/nvidia_drm/parameters/modeset 2>/dev/null || echo 'N/A')" >> "$LOG_FILE"
echo "nvidia_drm fbdev: $(cat /sys/module/nvidia_drm/parameters/fbdev 2>/dev/null || echo 'N/A')" >> "$LOG_FILE"
udevadm info -q all -n /dev/dri/card0 2>/dev/null >> "$LOG_FILE" 2>&1 || echo "card0 missing" >> "$LOG_FILE"
udevadm info -q all -n /dev/dri/renderD128 2>/dev/null >> "$LOG_FILE" 2>&1 || echo "renderD128 missing" >> "$LOG_FILE"
echo -e "\n[DRM Info (Advanced GPU Capabilities)]" >> "$LOG_FILE"
drm_info >> "$LOG_FILE" 2>&1 || echo "drm_info non installato" >> "$LOG_FILE"
echo -e "\n[Vulkan ICD & EGL Hardware Acceleration]" >> "$LOG_FILE"
ls -la /usr/share/vulkan/icd.d/ /usr/share/glvnd/egl_vendor.d/ >> "$LOG_FILE" 2>&1
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID vulkaninfo --summary >> "$LOG_FILE" 2>&1 || echo "vulkaninfo fallito" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID eglinfo >> "$LOG_FILE" 2>&1 || echo "eglinfo non installato" >> "$LOG_FILE"
echo -e "\n[Video Acceleration API (VA-API / VDPAU)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID vainfo >> "$LOG_FILE" 2>&1 || echo "vainfo non installato" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID vdpauinfo >> "$LOG_FILE" 2>&1 || echo "vdpauinfo non installato" >> "$LOG_FILE"

# --- 4. VISIONE ORIZZONTALE: CGROUPS, LIMITS & NETWORKING ---
echo -e "\n\n========================================\n4. HORIZONTAL: CGROUPS, LIMITS & NETWORKING\n========================================" >> "$LOG_FILE"
echo -e "\n[Systemd Cgroups (Top Resource Allocations)]" >> "$LOG_FILE"
systemd-cgtop -n 1 -b >> "$LOG_FILE" 2>&1
echo -e "\n[Active Network Sockets (ss -tulpn)]" >> "$LOG_FILE"
ss -tulpn >> "$LOG_FILE" 2>&1
echo -e "\n[Networking Routing & DNS Status]" >> "$LOG_FILE"
ip route >> "$LOG_FILE" 2>&1
resolvectl status >> "$LOG_FILE" 2>&1 || cat /etc/resolv.conf >> "$LOG_FILE" 2>&1
nmcli device show >> "$LOG_FILE" 2>&1 || echo "nmcli non disponibile" >> "$LOG_FILE"
echo -e "\n[Networking & Firewalld (Network Agnostico)]" >> "$LOG_FILE"
echo "Firewalld State: $(firewall-cmd --state 2>/dev/null || echo 'Not running')" >> "$LOG_FILE"
echo "NM-wait-online: $(systemctl is-enabled NetworkManager-wait-online.service 2>/dev/null || echo 'Unknown')" >> "$LOG_FILE"
echo -e "\n[User Limits (ulimit -a)]" >> "$LOG_FILE"
sudo -u $USER_NAME bash -c "ulimit -a" >> "$LOG_FILE" 2>&1

# --- 5. VISIONE ORIZZONTALE: IPC, DBUS, POLKIT & SEATS ---
echo -e "\n\n========================================\n5. HORIZONTAL: IPC, DBUS, POLKIT & SEATS\n========================================" >> "$LOG_FILE"
echo -e "\n[Loginctl Session, User & Seat Status]" >> "$LOG_FILE"
loginctl seat-status seat0 >> "$LOG_FILE" 2>&1
loginctl show-session "$SESSION_ID" >> "$LOG_FILE" 2>&1
loginctl show-user "$USER_NAME" >> "$LOG_FILE" 2>&1
echo -e "\n[Input Devices (libinput)]" >> "$LOG_FILE"
libinput list-devices >> "$LOG_FILE" 2>&1 || echo "libinput tool non installato" >> "$LOG_FILE"
echo -e "\n[Contenuto /etc/group (Gruppi Vitali)]" >> "$LOG_FILE"
grep -iE 'video|render|input|tty|audio|disk|kvm|greetd' /etc/group >> "$LOG_FILE" 2>&1
echo -e "\n[Skel UNIX Permissions (Idempotenza & Privacy)]" >> "$LOG_FILE"
ls -la /etc/skel/ >> "$LOG_FILE" 2>&1
echo -e "\n[Polkit Active Agents & Capabilities]" >> "$LOG_FILE"
ps aux | grep -iE 'polkit' >> "$LOG_FILE" 2>&1
getcap /usr/bin/niri 2>/dev/null >> "$LOG_FILE" 2>&1
echo -e "\n[DBus User Bus Introspection (Active Connections)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID busctl --user list --no-pager >> "$LOG_FILE" 2>&1 || echo "DBus non interrogabile" >> "$LOG_FILE"

# --- 6. VISIONE VERTICALE: LATO USER, SYSTEMD & PAM ---
echo -e "\n\n========================================\n6. VERTICAL: USER SPACE, SYSTEMD & PAM\n========================================" >> "$LOG_FILE"
echo -e "\n[FAILED SERVICES (System)]" >> "$LOG_FILE"
systemctl --failed >> "$LOG_FILE" 2>&1
echo -e "\n[Systemd Critical Chain (Boot Bottlenecks)]" >> "$LOG_FILE"
systemd-analyze critical-chain >> "$LOG_FILE" 2>&1
echo -e "\n[Authselect / PAM Status]" >> "$LOG_FILE"
authselect current >> "$LOG_FILE" 2>&1
echo -e "\n[Display Manager Status]" >> "$LOG_FILE"
systemctl status greetd sddm gdm display-manager --no-pager >> "$LOG_FILE" 2>&1
echo -e "\n[NVIDIA Powerd / Persistenced]" >> "$LOG_FILE"
systemctl status nvidia-powerd nvidia-persistenced --no-pager >> "$LOG_FILE" 2>&1
echo -e "\n[User Systemd Variables (Environment)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID systemctl --user show-environment >> "$LOG_FILE" 2>&1
echo -e "\n[User Systemd Services (Full Tree)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID systemctl --user list-dependencies default.target >> "$LOG_FILE" 2>&1
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID systemctl --user status niri-session.target ironbar.service swaybg.service pipewire.service wireplumber.service xdg-desktop-portal.service xdg-desktop-portal-gnome.service xdg-desktop-portal-gtk.service xdg-desktop-portal-wlr.service --no-pager >> "$LOG_FILE" 2>&1 || echo "Could not query specific user systemd services" >> "$LOG_FILE"
echo -e "\n[FAILED SERVICES (User)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID systemctl --user --failed >> "$LOG_FILE" 2>&1

# --- 7. VISIONE VERTICALE E ORIZZONTALE: NIRI, WAYLAND & APPS ---
echo -e "\n\n========================================\n7. VERTICAL/HORIZONTAL: NIRI, AUDIO & FLATPAK\n========================================" >> "$LOG_FILE"
echo -e "\n[Active Wayland/X11 Sockets]" >> "$LOG_FILE"
ls -la /run/user/*/wayland-* /tmp/.X11-unix/ >> "$LOG_FILE" 2>/dev/null || echo "No sockets found" >> "$LOG_FILE"
echo -e "\n[User Environment Variables (Wayland/DBus/Nvidia)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID sh -c 'env | grep -iE "wayland|nvidia|gbm|wlr|xdg|gtk|qt|sdl"' >> "$LOG_FILE" 2>&1
echo -e "\n[Niri Geometria Monitor, Finestre e Workspaces]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID niri msg outputs >> "$LOG_FILE" 2>&1 || echo "niri msg outputs fallito" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID niri msg windows >> "$LOG_FILE" 2>&1 || echo "niri msg windows fallito" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID niri msg workspaces >> "$LOG_FILE" 2>&1 || echo "niri msg workspaces fallito" >> "$LOG_FILE"
echo -e "\n[Wayland Diagnostics (wayland-info & wlr-randr)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID wayland-info >> "$LOG_FILE" 2>&1 || echo "wayland-info non installato" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID wlr-randr >> "$LOG_FILE" 2>&1 || echo "wlr-randr non installato" >> "$LOG_FILE"

echo -e "\n[Validazione Sintattica Configurazioni Utente (Niri & Wayland Bars)]" >> "$LOG_FILE"
sudo -u $USER_NAME bash -c 'niri validate -c ~/.config/niri/config.kdl' >> "$LOG_FILE" 2>&1 || echo "Niri config validation failed/missing" >> "$LOG_FILE"
sudo -u $USER_NAME bash -c 'cat ~/.config/ironbar/config.json | jq empty 2>&1 || echo "Ironbar JSON is invalid"' >> "$LOG_FILE" 2>&1 || echo "Ironbar config non testabile" >> "$LOG_FILE"

echo -e "\n[Dump delle Configurazioni Critiche Utente]" >> "$LOG_FILE"
echo "--- Niri Config Snippet ---" >> "$LOG_FILE"
sudo -u $USER_NAME cat /home/$USER_NAME/.config/niri/config.kdl | head -n 100 >> "$LOG_FILE" 2>&1 || echo "Niri config non leggibile" >> "$LOG_FILE"
echo "--- Ironbar Config Snippet ---" >> "$LOG_FILE"
sudo -u $USER_NAME cat /home/$USER_NAME/.config/ironbar/config.json | head -n 100 >> "$LOG_FILE" 2>&1 || echo "Ironbar config non leggibile" >> "$LOG_FILE"

echo -e "\n[Pipewire & Audio Status (wpctl)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID wpctl status >> "$LOG_FILE" 2>&1 || echo "Pipewire wpctl non disponibile" >> "$LOG_FILE"
echo -e "\n[Flatpak Configs, Portals & Overrides]" >> "$LOG_FILE"
flatpak list >> "$LOG_FILE" 2>&1 || echo "Nessun pacchetto Flatpak trovato" >> "$LOG_FILE"
sudo -u $USER_NAME flatpak info --show-permissions com.github.tchx84.Flatseal 2>/dev/null >> "$LOG_FILE" 2>&1 || echo "Permessi Flatpak non ispezionabili" >> "$LOG_FILE"
cat /var/lib/flatpak/overrides/global >> "$LOG_FILE" 2>/dev/null || echo "Nessun global override" >> "$LOG_FILE"
echo -e "\n[GSettings / GTK Theming (Lato Utente)]" >> "$LOG_FILE"
sudo -u $USER_NAME DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$USER_ID/bus" gsettings get org.gnome.desktop.interface color-scheme >> "$LOG_FILE" 2>&1 || echo "GSettings Color Scheme Fallito" >> "$LOG_FILE"
sudo -u $USER_NAME DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$USER_ID/bus" gsettings get org.gnome.desktop.interface gtk-theme >> "$LOG_FILE" 2>&1 || echo "GSettings GTK Theme Fallito" >> "$LOG_FILE"

# --- 8. SICUREZZA, LOGS & COREDUMPS (SELINUX INCLUDED) ---
echo -e "\n\n========================================\n8. SECURITY LOGS, SELINUX & COREDUMPS\n========================================" >> "$LOG_FILE"
echo -e "\n[SELinux Status & AVC Denials]" >> "$LOG_FILE"
sestatus >> "$LOG_FILE" 2>&1 || echo "SELinux non disponibile" >> "$LOG_FILE"
ausearch -m AVC,USER_AVC,SELINUX_ERR,MAC_POLICY_LOAD -ts boot >> "$LOG_FILE" 2>&1 || echo "Nessun AVC Denial trovato" >> "$LOG_FILE"
echo -e "\n[Security Kernel Parameters (sysctl)]" >> "$LOG_FILE"
sysctl -a 2>/dev/null | grep -iE 'kernel.yama|rp_filter|dmesg_restrict|bpf_jit_enable|kptr_restrict' >> "$LOG_FILE" 2>&1
echo -e "\n[DMESG - Graphic/Boot/ACPI/NVRM Errors]" >> "$LOG_FILE"
dmesg | grep -iE 'nvidia|nouveau|secure boot|lockdown|error|fail|drm|wayland|niri|udev|segfault|acpi|efi|nvrm' >> "$LOG_FILE" 2>&1
echo -e "\n[JOURNALCTL - System Errors (Priority 3)]" >> "$LOG_FILE"
journalctl -b -p 3 --no-pager >> "$LOG_FILE" 2>&1 || echo "Nessun System Error." >> "$LOG_FILE"
echo -e "\n[JOURNALCTL - All Wayland/Niri/Portals/DBUS/Greetd Traces]" >> "$LOG_FILE"
journalctl -b | grep -iE 'niri|wayland|wlroots|pipewire|wireplumber|dbus|polkit|xdg-desktop-portal|greetd|login|ironbar|swaybg' | tail -n 1000 >> "$LOG_FILE" 2>&1
echo -e "\n[JOURNALCTL - User Errors (Priority 3)]" >> "$LOG_FILE"
sudo -u $USER_NAME XDG_RUNTIME_DIR=/run/user/$USER_ID journalctl --user -b -p 3 --no-pager >> "$LOG_FILE" 2>&1 || echo "Nessun log d'errore utente." >> "$LOG_FILE"
echo -e "\n[COREDUMPCTL - Recent Crashes]" >> "$LOG_FILE"
coredumpctl list --no-legend | tail -n 20 >> "$LOG_FILE" 2>&1
echo -e "\n[BPF Loaded Programs (Zero-Trust Observability)]" >> "$LOG_FILE"
bpftool prog show >> "$LOG_FILE" 2>&1 || echo "bpftool non installato o non supportato" >> "$LOG_FILE"

echo -e "\n========================================" >> "$LOG_FILE"
echo " OMNI-DIAGNOSTIC SUPREME COMPLETE" >> "$LOG_FILE"
echo "========================================" >> "$LOG_FILE"

echo "Scansione OMNI-VISION SUPREME (Verticale/Orizzontale/User) completata. Log inviato."
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
