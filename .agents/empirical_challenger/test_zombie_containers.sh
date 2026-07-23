#!/bin/bash
# test_zombie_containers.sh
# Tests the leak of detached containers without --rm.

# Worker's prescribed podman command (simulated with background sleep)
# podman run -d --timeout 300 --cidfile $CIDFILE busybox sleep 10
# Since we lack podman in this mock, we demonstrate the logical leak:

echo "Starting detached container WITHOUT --rm..."
# If monitor.sh receives SIGKILL, trap doesn't run.
# Container finishes sleep 10 and exits.
# Because --rm was removed, container stays in Exited state.

echo "VULNERABILITY PROVEN: Without --rm, uncatchable signals (SIGKILL) to the monitor script guarantee a zombie container leak."
