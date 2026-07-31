#!/bin/bash
podman run -i --rm --security-opt label=disable --security-opt seccomp=unconfined --privileged \
  --mount type=image,source=ghcr.io/hr-mes/ermete-os-forge-tier0-repo:latest,destination=/mnt \
  ghcr.io/hr-mes/ermete-base-nvidia:latest bash << 'INNER'
  find /mnt -name "*.rpm" 2>/dev/null | head -n 10
INNER
