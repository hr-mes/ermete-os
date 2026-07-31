#!/bin/bash
ctr=$(podman create ghcr.io/hr-mes/ermete-os-forge-tier0-repo:latest)
mkdir -p /tmp/test-dnf7
podman cp $ctr:/ermete-kernel-devel-7.0.0~rc5-1.chimera.fc44.x86_64.rpm /tmp/test-dnf7/
podman cp $ctr:/kernel-devel-6.14.5-100.chimera.fc43.x86_64.rpm /tmp/test-dnf7/
podman cp $ctr:/kernel-devel-matched-6.14.5-100.chimera.fc43.x86_64.rpm /tmp/test-dnf7/
podman cp $ctr:/kernel-core-6.14.5-100.chimera.fc43.x86_64.rpm /tmp/test-dnf7/
podman cp $ctr:/kernel-6.14.5-100.chimera.fc43.x86_64.rpm /tmp/test-dnf7/
podman cp $ctr:/kernel-modules-6.14.5-100.chimera.fc43.x86_64.rpm /tmp/test-dnf7/
podman cp $ctr:/kernel-modules-core-6.14.5-100.chimera.fc43.x86_64.rpm /tmp/test-dnf7/
podman cp $ctr:/kernel-modules-extra-6.14.5-100.chimera.fc43.x86_64.rpm /tmp/test-dnf7/
# Run podman bash and try to install all of them at once
podman run -i --rm --security-opt label=disable --security-opt seccomp=unconfined --privileged \
  -v /tmp/test-dnf7:/mnt ghcr.io/hr-mes/ermete-base-nvidia:latest bash << 'INNER'
  dnf5 remove -y kernel-devel kernel-headers kernel
  RPM_LIST=$(find /mnt -name "*.rpm")
  echo "Installing: $RPM_LIST"
  dnf5 install -y --allowerasing --setopt=install_weak_deps=False --setopt=tsflags=nodocs $RPM_LIST
INNER
