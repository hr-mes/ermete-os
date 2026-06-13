#!/bin/bash
set -ouex pipefail

echo "--- Applying Private Power-User Optimizations ---"

# 1. Installazione dipendenze
dnf -y install --setopt=install_weak_deps=False greenboot fwupd

# 2. Configurazione Greenboot (Auto-Riparazione)
mkdir -p /etc/greenboot/check/required.d/

# Script 01: Controllo Rete (Se fallisce 3 volte, fa scattare il rollback ostree)
cat > /etc/greenboot/check/required.d/01-network-check.sh << 'EOF'
#!/bin/bash
# Verifica semplicemente che il demone di rete non sia andato in crash.
# Evitiamo i ping esterni (es. 1.1.1.1) per non innescare falsi positivi
# in caso di avvio offline intenzionale, assenza Wi-Fi o captive portal in hotel/aeroporto.
if systemctl is-active --quiet NetworkManager.service; then
    exit 0
else
    exit 1
fi
EOF
chmod +x /etc/greenboot/check/required.d/01-network-check.sh

# Script 02: Controllo Greetd (Se fallisce, Wayland/GUI rotti -> rollback)
cat > /etc/greenboot/check/required.d/02-greetd-check.sh << 'EOF'
#!/bin/bash
if systemctl is-active --quiet greetd.service; then
    exit 0
else
    exit 1
fi
EOF
chmod +x /etc/greenboot/check/required.d/02-greetd-check.sh

# 3. Manutenzione Automatica (fstrim e fwupd)
echo "enable fstrim.timer" >> /usr/lib/systemd/system-preset/99-Ermete.preset
echo "enable fwupd.service" >> /usr/lib/systemd/system-preset/99-Ermete.preset

# 4. First-boot Service per installare Flatseal e flatpaks essenziali
# Sfrutta un lockfile in /var/lib per assicurarsi che venga eseguito con successo almeno una volta,
# anche se l'utente si connette al Wi-Fi in un secondo momento, senza bloccare il boot!
mkdir -p /usr/libexec/
cat > /usr/libexec/ermete-firstboot.sh << 'EOF'
#!/bin/bash
set -ouex pipefail

# Fail se non c'è rete per delegare il retry a systemd
curl -s -f --connect-timeout 10 "https://dl.flathub.org/repo/flathub.flatpakrepo" > /dev/null || exit 1

if flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo && \
   flatpak override --system --env=GTK_THEME=adw-gtk3-dark && \
   flatpak override --system --env=ICON_THEME=Papirus-Dark && \
   flatpak install -y flathub io.github.flattool.Warehouse com.github.tchx84.Flatseal org.gnome.Nautilus io.mpv.Mpv com.obsproject.Studio com.github.wwmm.easyeffects org.mozilla.firefox; then
  touch /var/lib/ermete-firstboot-done
else
  exit 1
fi
EOF
chmod +x /usr/libexec/ermete-firstboot.sh

cat > /etc/systemd/system/Ermete-firstboot.service << 'EOF'
[Unit]
Description=Ermete First Boot Setup (Install Flatpaks)
After=network-online.target
Wants=network-online.target
ConditionPathExists=!/var/lib/ermete-firstboot-done

[Service]
Type=oneshot
ExecStart=/usr/libexec/ermete-firstboot.sh
Restart=on-failure
RestartSec=30

[Install]
WantedBy=multi-user.target
EOF
echo "enable Ermete-firstboot.service" >> /usr/lib/systemd/system-preset/99-Ermete.preset

# 5. Pulizia Visiva del Kernel (Silent Boot) tramite bootc kargs
mkdir -p /usr/lib/bootc/kargs.d/
cat > /usr/lib/bootc/kargs.d/10-silent.toml << 'EOF'
kargs = ["quiet", "loglevel=3", "rd.udev.log_level=3", "vt.global_cursor_default=0"]
EOF

echo "--- Private Optimizations Applied ---"
