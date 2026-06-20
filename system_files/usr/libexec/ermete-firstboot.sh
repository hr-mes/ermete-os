#!/bin/bash
set -ouex pipefail

# 1. FIX CONFIGURATION PERMISSIONS FOR ALL USERS
# Independent of network state, we ensure that if /home/*/.config exists,
# it belongs to the respective user. This is a robust fallback for provisioning bugs.
for user_home in /home/*; do
  if [ -d "$user_home/.config" ]; then
    user=$(basename "$user_home")
    # Prevent hardcoding UID 1000
    uid=$(stat -c "%u" "$user_home")
    gid=$(stat -c "%g" "$user_home")
    chown -R "$uid:$gid" "$user_home/.config" || true
  fi
done

# 2. ASYNC PROVISIONING (FLATPAKS)
if curl -s -f --connect-timeout 10 "https://dl.flathub.org/repo/flathub.flatpakrepo" > /dev/null; then
  if flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo && \
     flatpak override --system --env=GTK_THEME=adw-gtk3-dark && \
     flatpak override --system --env=ICON_THEME=Papirus-Dark && \
     flatpak override --system --env=XCURSOR_THEME=Bibata-Modern-Classic && \
     flatpak override --system --env=XCURSOR_SIZE=24 && \
     flatpak override --system --socket=wayland --socket=fallback-x11 --device=dri && \
     flatpak install -y --noninteractive flathub org.gtk.Gtk3theme.adw-gtk3 org.gtk.Gtk3theme.adw-gtk3-dark io.github.flattool.Warehouse com.github.tchx84.Flatseal com.obsproject.Studio com.github.wwmm.easyeffects org.mozilla.firefox; then
    
    touch /var/lib/ermete-firstboot-done
  else
    exit 1
  fi
else
  exit 1
fi
