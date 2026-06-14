#!/bin/bash
set -ouex pipefail
if systemctl is-active --quiet greetd.service; then
    logger -t greenboot "Greetd UI check PASSED"
    exit 0
else
    logger -t greenboot "Greetd UI check FAILED - Triggering rollback"
    exit 1
fi
