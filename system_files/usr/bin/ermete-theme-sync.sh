#!/bin/bash
set -e

if [ -z "$1" ]; then
    echo "Usage: ermete-theme-sync.sh <path_to_image>"
    exit 1
fi

IMAGE_PATH="$1"
IMAGE_PATH=$(realpath "$IMAGE_PATH")

if [ ! -f "$IMAGE_PATH" ]; then
    echo "File not found: $IMAGE_PATH"
    exit 1
fi

# 1. Update background gracefully (prevent black screen flicker)
swaybg -i "$IMAGE_PATH" -m fill &
NEW_BG_PID=$!
sleep 0.5
for pid in $(pgrep -x swaybg); do
    if [ "$pid" != "$NEW_BG_PID" ]; then
        kill "$pid" || true
    fi
done

# 2. Run Matugen to generate colors and templates (GTK4 is handled automatically)
matugen image "$IMAGE_PATH"

# 3. Reload Niri config (since matugen wrote the KDL from the template)
niri msg action do-screen-transition || true

# 4. Reload Waybar (SIGUSR2 reloads styles without killing the process)
pkill -SIGUSR2 waybar || true

# 5. Reload foot terminals
pkill -SIGUSR1 foot || true

# 6. Reload mako (if used)
makoctl reload || true

echo "Theme successfully synced!"
