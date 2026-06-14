#!/usr/bin/env bash
# ==============================================================================
# ERMETE OS - OMNI-VISION SUPREME v9.0 (ZERO-TRUST X-RAY)
# Architettura OCI Enterprise-Grade | Read-Only Execution
# ==============================================================================
set -euo pipefail

LOGFILE="/tmp/ermete_omni_vision_$(date +%Y%m%d_%H%M%S).log"
exec > >(tee -i "$LOGFILE") 2>&1

echo "========================================"
echo " 🦅 ERMETE OS - OMNI-VISION SUPREME v9.0"
echo " Timestamp: $(date --rfc-3339=seconds)"
echo "========================================"

echo -e "\n[+] LAYER 0: OCI INTEGRITY & CRYPTOGRAPHIC ANCHORS"
echo "------------------------------------------------"
echo ">>> Bootc Status & Image Signature:"
bootc status || echo "Bootc non disponibile"
echo ">>> OSTree Mutated Configs (/etc Diffing):"
sudo ostree admin config-diff || echo "Nessuna deviazione in /etc o errore OSTree."
echo ">>> DNF / RPM Atomic Immutable Tree:"
rpm-ostree status -v
echo ">>> GPG Trust Anchors (Local):"
ls -lZ /etc/pki/rpm-gpg/ || true

echo -e "\n[+] LAYER 0: KERNEL, SELINUX & EBPF SECURITY"
echo "------------------------------------------------"
echo ">>> SELinux Enforcing Status & Booleans:"
sestatus -v
sudo getsebool -a | grep -E "wayland|nvidia|execmem|mmap_zero" || true
echo ">>> Recent SELinux AVC Denials (Letali):"
sudo ausearch -m AVC,USER_AVC -ts recent --raw | audit2allow || echo "Nessun AVC recente."
echo ">>> eBPF Loaded Programs (Kernel Tracing):"
sudo bpftool prog show || echo "bpftool assente o non supportato. Aggiungere 'bpftool' al Containerfile base."

echo -e "\n[+] LAYER 0: HARDWARE, DRM & NVIDIA GBM"
echo "------------------------------------------------"
echo ">>> Kernel Kargs (Modesetting & IOMMU):"
cat /proc/cmdline
echo ">>> NVIDIA DKMS & Modprobe Status:"
lsmod | grep -iE "nvidia|nouveau"
echo ">>> DRM Pipelines & Nodes:"
ls -lZ /dev/dri/
drm_info || echo "drm_info non installato. (Consigliato per debug Wayland/NVIDIA)."

echo -e "\n[+] LAYER 1: WAYLAND & SYSTEMD USER LIFECYCLE"
echo "------------------------------------------------"
echo ">>> Systemd-User Failed Units (Sessione Utente):"
systemctl --user --state=failed
echo ">>> Systemd-User Active Targets (Niri, Pipewire, Portal):"
systemctl --user status niri-session.target pipewire.service xdg-desktop-portal.service --no-pager || true
echo ">>> DBus Session Tree (Interfacce Registrate):"
busctl --user tree org.freedesktop.portal.Desktop || echo "XDG Portal non agganciato."
echo ">>> Polkit Agent Authentication:"
pgrep -lfa polkit || echo "ATTENZIONE: Nessun Polkit Agent in esecuzione!"

echo -e "\n[+] LAYER 1: PRIVACY SANDBOXING & UX IDEMPOTENCY"
echo "------------------------------------------------"
echo ">>> Skel & Home Directory Permissions (Strict 700/600):"
stat -c "%A %U:%G %n" /etc/skel /var/home/$USER ~/.config/niri/config.kdl ~/.config/ironbar/config.json || true
echo ">>> Wayland Environment Check:"
env | grep -iE "XDG_SESSION_TYPE|WAYLAND_DISPLAY|WLR_NO_HARDWARE_CURSORS|GBM_BACKEND|MOZ_ENABLE_WAYLAND" || true

echo -e "\n[+] HORIZONTAL: ZERO-TRUST NETWORK POSTURE"
echo "------------------------------------------------"
echo ">>> Nftables Pure Ruleset (Silently Dropping):"
sudo nft list ruleset || echo "nftables disabilitato o non in uso."
echo ">>> DNS-over-TLS & Link Status:"
resolvectl status | grep -E "DNS|Protocols" || true

echo -e "\n========================================"
echo " X-RAY COMPLETATA. Log salvato in: $LOGFILE"
echo "========================================"
