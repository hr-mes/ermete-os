#!/bin/bash
ctr=$(podman create ghcr.io/hr-mes/ermete-os-forge-tier0-repo:latest)
podman export $ctr | tar -tv | awk '{print $6}' | grep "\.rpm$" | grep -vE 'systemd-standalone-|.*-free-.*\.rpm|glibc32|kernel-debug|rpm-.*\.rpm' > filtered.txt
echo "Total RPMs after filter:"
wc -l filtered.txt
echo "Checking for kernel-devel in filtered:"
grep kernel-devel filtered.txt
