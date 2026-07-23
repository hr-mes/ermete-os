#!/bin/bash
systemd-run --user --scope --property=BindsTo=dbus.service podman run -d --name test_sysd_run alpine sleep 1000
echo "Podman run detached completed."
sleep 2
podman ps -f name=test_sysd_run
podman rm -f test_sysd_run
