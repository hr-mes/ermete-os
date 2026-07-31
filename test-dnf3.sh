#!/bin/bash
podman run -i --rm --security-opt label=disable --security-opt seccomp=unconfined --privileged ghcr.io/hr-mes/ermete-os-builder:latest bash << 'INNER'
  mkdir -p /tmp/rpm
  dnf5 download --destdir=/tmp/rpm -y libgpg-error
  rpm=$(ls /tmp/rpm/*x86_64.rpm | head -n 1)
  echo "Running dnf5 upgrade on $rpm"
  dnf5 upgrade -y $rpm || echo "FAILED with $?"
  echo "Running dnf5 install on $rpm"
  dnf5 install -y $rpm || echo "FAILED with $?"
INNER
