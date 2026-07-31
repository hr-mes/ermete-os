#!/bin/bash
ctr=$(podman create ghcr.io/hr-mes/ermete-os-forge-tier0-repo:latest)
podman cp $ctr:/ermete-kernel-devel-7.0.0~rc5-1.chimera.fc44.x86_64.rpm ./ermete-kernel-devel.rpm
podman cp $ctr:/kernel-devel-6.14.5-100.chimera.fc43.x86_64.rpm ./kernel-devel.rpm
rpm -qp --provides ./ermete-kernel-devel.rpm
echo "---"
rpm -qp --provides ./kernel-devel.rpm
