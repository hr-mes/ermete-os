#!/bin/bash
set -e

for i in {1..15}; do
    if systemctl is-active --quiet greetd.service; then
        echo "Greenboot check: greetd is running."
        exit 0
    fi
    sleep 1
done

echo "Greenboot check: greetd is NOT running."
exit 1
