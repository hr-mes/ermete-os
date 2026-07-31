#!/bin/bash
podman run -i --rm --security-opt label=disable --security-opt seccomp=unconfined --privileged ghcr.io/hr-mes/ermete-base-nvidia:latest bash << 'INNER'
  mkdir -p /tmp/rpm
  dnf5 download --destdir=/tmp/rpm -y wget
  rpm=$(ls /tmp/rpm/wget*.rpm | head -n 1)
  echo "Running dnf5 upgrade on a NEW package: $rpm"
  dnf5 upgrade -y $rpm || echo "FAILED with $?"
  rpm -q wget
INNER
