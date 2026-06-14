#!/bin/bash
if systemctl is-active --quiet NetworkManager.service; then
    exit 0
else
    exit 1
fi
