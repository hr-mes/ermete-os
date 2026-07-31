#!/bin/bash
podman run -i --rm --security-opt label=disable --security-opt seccomp=unconfined --privileged ghcr.io/hr-mes/ermete-base-nvidia:latest bash << 'INNER'
  mkdir -p /tmp/rpm
  # Download older version of kernel-devel
  dnf5 download --destdir=/tmp/rpm -y bash
  # We already have bash installed. Let's see if dnf5 install upgrades/downgrades it or skips it.
  # Well, bash is already the latest. We need something we can downgrade, or we can just test the output.
  echo "Just checking what dnf5 install does on a package that is already installed"
INNER
