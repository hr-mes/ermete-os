#!/bin/bash
set -ouex pipefail
curl -s -f --connect-timeout 10 "https://dl.flathub.org/repo/flathub.flatpakrepo" > /dev/null || exit 1
if flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo && \
   flatpak override --system --env=GTK_THEME=adw-gtk3-dark && \
   flatpak override --system --env=ICON_THEME=Papirus-Dark && \
   flatpak override --system --env=XCURSOR_THEME=Bibata-Modern-Classic && \
   flatpak override --system --env=XCURSOR_SIZE=24 && \
   flatpak override --system --socket=wayland --socket=fallback-x11 --device=dri && \
   flatpak install -y --noninteractive flathub org.gtk.Gtk3theme.adw-gtk3 org.gtk.Gtk3theme.adw-gtk3-dark io.github.flattool.Warehouse com.github.tchx84.Flatseal com.obsproject.Studio com.github.wwmm.easyeffects org.mozilla.firefox; then
  
  # Fix configuration permissions for user ermete so Ironbar and Alacritty can start
  chown -R 1000:1000 /home/ermete/.config || true
  
  touch /var/lib/ermete-firstboot-done
else
  exit 1
fi
