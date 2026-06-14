#!/bin/bash
set -ouex pipefail
# Test crittografico di connettività verso un resolver esterno (timeout 10s)
if curl -s -f --connect-timeout 10 https://1.1.1.1 > /dev/null 2>&1; then
    logger -t greenboot "Network check PASSED"
    exit 0
else
    logger -t greenboot "Network check FAILED - Triggering rollback"
    exit 1
fi
