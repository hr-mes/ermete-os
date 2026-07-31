#!/bin/bash
podman run -i --rm --security-opt label=disable --security-opt seccomp=unconfined --privileged ghcr.io/hr-mes/ermete-os-builder:latest bash << 'INNER'
  mkdir -p /tmp/rpm
  dnf5 download --destdir=/tmp/rpm -y libgpg-error
  rpm=$(ls /tmp/rpm/*.rpm | head -n 1)
  pkg_name=$(rpm -qp --queryformat "%{NAME}\n" "$rpm" 2>/dev/null || true)
  echo "PKG_NAME='$pkg_name'"
  if rpm -q "$pkg_name" >/dev/null 2>&1; then
      echo "INSTALLED"
  else
      echo "NOT INSTALLED"
  fi
INNER
