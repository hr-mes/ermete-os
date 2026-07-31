#!/bin/bash
podman run -i --rm --security-opt label=disable --security-opt seccomp=unconfined --privileged \
  -v /tmp/tier0-repo-test2:/opt/tier0-repo ghcr.io/hr-mes/ermete-base-nvidia:latest bash << 'INNER'
  dnf5 remove -y kernel-devel kernel-headers kernel
  rpm -e --nodeps fedora-logos || true
  RPM_LIST=$(find /opt/tier0-repo -name "*.rpm" 2>/dev/null | grep -vE 'systemd-standalone-|.*-free-.*\.rpm|glibc32|kernel-debug|rpm-.*\.rpm' || true)
  echo "Installing $(echo "$RPM_LIST" | wc -l) RPMs..."
  dnf5 install -y --allowerasing --setopt=install_weak_deps=False --setopt=tsflags=nodocs $RPM_LIST
INNER
