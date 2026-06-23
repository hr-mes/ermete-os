#!/bin/bash
set -euo pipefail

STATE_FILE="$HOME/.local/state/ermete-ide-bootstrapped"

# Se l'IDE è già stato bootstrappato, usciamo immediatamente.
if [ -f "$STATE_FILE" ]; then
    exit 0
fi

# Aspettiamo che la rete sia disponibile e che Nix sia pronto.
# Utilizziamo un ping leggero a cache.nixos.org.
echo "Attesa connettività verso cache.nixos.org..."
until curl -sI https://cache.nixos.org | grep -q "200"; do
    sleep 5
done

echo "Connettività stabilita. Avvio bootstrap IDE tramite Nix..."
notify-send "Ermete OS" "Download IDE Stack in corso (gcc, make, lazygit)..." -i software-update-available -u normal

# Assicuriamoci che nix sia accessibile per il task asincrono
if [ -f /etc/profile.d/nix.sh ]; then
    source /etc/profile.d/nix.sh
fi

if nix --extra-experimental-features "nix-command flakes" profile install nixpkgs#gcc nixpkgs#gnumake nixpkgs#lazygit nixpkgs#nodejs_22; then
    mkdir -p "$(dirname "$STATE_FILE")"
    touch "$STATE_FILE"
    notify-send "Ermete OS" "IDE Stack installato con successo! LazyVim è pronto." -i emblem-default -u normal
else
    notify-send "Ermete OS" "Errore durante il download dell'IDE Stack via Nix." -i dialog-error -u critical
    exit 1
fi
