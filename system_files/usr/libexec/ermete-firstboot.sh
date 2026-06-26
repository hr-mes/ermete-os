#!/bin/bash
set -ouex pipefail

# 1. PROVISIONING CONFIGURAZIONI (SKEL) E FIX PERMESSI PER UTENTI ESISTENTI
# Garantisce che il passaggio da altri ambienti atomic (es. Kinoite) ad Ermete OS 
# inietti i file di configurazione corretti di Niri senza sovrascrivere file esistenti.
for user_home in /home/*; do
  if [ -d "$user_home" ]; then
    user=$(basename "$user_home")
    # Prevent hardcoding UID 1000
    uid=$(stat -c "%u" "$user_home")
    gid=$(stat -c "%g" "$user_home")
    
    # Clona i file di skel mancanti (no-clobber)
    cp -rn /etc/skel/. "$user_home/" || true
    
    # Ripristina la proprietà
    chown -R "$uid:$gid" "$user_home/.config" "$user_home/Pictures" 2>/dev/null || true
  fi
done

# 2. ASYNC PROVISIONING (FLATPAKS)
# Wait for internet connectivity resiliently (Zero-Boot-Delay architectural constraint)
max_retries=12
retries=0
until curl -s -f --connect-timeout 5 "https://dl.flathub.org/repo/flathub.flatpakrepo" > /dev/null; do
  echo "Attendendo uplink di rete per provisioning Flatpak..."
  sleep 5
  retries=$((retries+1))
  if [ "$retries" -ge "$max_retries" ]; then
    echo "Timeout di rete superato. Uscita per fall-back a systemd restart."
    exit 1
  fi
done

if flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo && \
   flatpak override --system --env=GTK_THEME=adw-gtk3-dark && \
   flatpak override --system --env=ICON_THEME=Papirus-Dark && \
   flatpak override --system --env=XCURSOR_THEME=Bibata-Modern-Classic && \
   flatpak override --system --env=XCURSOR_SIZE=24 && \
   flatpak override --system --socket=wayland --socket=fallback-x11 --device=dri && \
   flatpak install -y --noninteractive flathub org.gtk.Gtk3theme.adw-gtk3 org.gtk.Gtk3theme.adw-gtk3-dark io.github.flattool.Warehouse com.github.tchx84.Flatseal com.obsproject.Studio com.github.wwmm.easyeffects org.mozilla.firefox org.xfce.thunar; then
  
  touch /var/lib/ermete-firstboot-done
else
  exit 1
fi
