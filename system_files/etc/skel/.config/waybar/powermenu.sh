#!/bin/bash

CHOICE=$(echo -e "  Shutdown\n  Reboot\n  Logout\n  Lock\n⏾  Suspend" | fuzzel --dmenu \
    --anchor=top-right \
    --x-margin=10 \
    --y-margin=8 \
    --lines=5 \
    --width=15 \
    --no-icons \
    --prompt="")

case "$CHOICE" in
    *"Shutdown") systemctl poweroff ;;
    *"Reboot") systemctl reboot ;;
    *"Logout") niri msg action quit ;;
    *"Lock") swaylock -c 000000 ;;
    *"Suspend") swaylock -f -c 000000 && systemctl suspend ;;
esac
