#!/bin/bash
set -ouex pipefail
curl -s -f --connect-timeout 10 "https://dl.flathub.org/repo/flathub.flatpakrepo" > /dev/null || exit 1
if flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo && \
   flatpak override --system --env=GTK_THEME=adw-gtk3-dark && \
   flatpak override --system --env=ICON_THEME=Papirus-Dark && \
   flatpak override --system --socket=wayland --socket=fallback-x11 --device=dri && \
   flatpak install -y flathub io.github.flattool.Warehouse com.github.tchx84.Flatseal org.gnome.Nautilus io.mpv.Mpv com.obsproject.Studio com.github.wwmm.easyeffects org.mozilla.firefox; then
  touch /var/lib/ermete-firstboot-done
else
  exit 1
fi
