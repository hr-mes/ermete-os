#!/bin/bash
set -e

if systemctl is-active --quiet greetd.service; then
    echo "Greenboot check: greetd is running."
    exit 0
else
    echo "Greenboot check: greetd is NOT running."
    exit 1
fi
