#!/bin/bash
set -euo pipefail

if [ "${0}" != "/usr/libexec/ermete-flatpak-provisioner" ]; then
    mkdir -p /usr/libexec
fi

MANIFEST="/usr/share/ermete/packages.json"
if [ ! -f "$MANIFEST" ]; then
    MANIFEST="/etc/ermete/packages.json"
fi

if [ ! -f "$MANIFEST" ]; then
    echo "[Ermete Flatpak] No package manifest found at $MANIFEST, skipping."
    exit 0
fi

if ! command -v flatpak &>/dev/null; then
    echo "[Ermete Flatpak] Flatpak binary not found, skipping."
    exit 0
fi

echo "[Ermete Flatpak] Configuring Flathub remote..."
flatpak remote-add --if-not-exists --system flathub https://dl.flathub.org/repo/flathub.flatpakrepo || true

FLATPAKS=$(jq -r '.flatpaks[]?' "$MANIFEST" 2>/dev/null || true)

if [ -z "$FLATPAKS" ]; then
    echo "[Ermete Flatpak] No flatpaks configured in manifest."
    exit 0
fi

for app in $FLATPAKS; do
    echo "[Ermete Flatpak] Provisioning $app..."
    if ! flatpak info "$app" &>/dev/null; then
        flatpak install --system -y --noninteractive flathub "$app" || {
            echo "[Ermete Flatpak] CRITICAL ERROR: Failed to install $app"
            exit 1
        }
    else
        echo "[Ermete Flatpak] $app is already installed."
    fi
done
