#!/bin/bash
podman run -i --rm --security-opt label=disable --security-opt seccomp=unconfined --privileged ghcr.io/hr-mes/ermete-base-nvidia:latest bash << 'INNER'
  mkdir -p /tmp/rpm
  # Download a newer version of bash, but there might not be one.
  # Let's downgrade bash using dnf5 downgrade, then test if dnf5 install local.rpm upgrades it.
  dnf5 download --destdir=/tmp/rpm -y wget
  rpm=$(ls /tmp/rpm/wget*.rpm | head -n 1)
  # Instead of downloading, we can just use any package that isn't installed.
  echo "Installing wget first..."
  dnf5 install -y wget
  echo "Downloading an older version of something?"
  # We just want to test if dnf5 install local.rpm UPGRADES.
  dnf5 download --destdir=/tmp/rpm -y libffi
  rpm2=$(ls /tmp/rpm/libffi*.rpm | head -n 1)
  echo "libffi is installed. Let's try dnf5 install on its rpm."
  dnf5 install -y $rpm2 || echo "FAILED"
INNER
