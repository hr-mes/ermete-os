#!/bin/bash
set -ouex pipefail

echo "--- Installing Desktop Environment ---"

# Install Niri 
dnf -y install niri bibata-cursor-theme

# Install Dank Linux shell
sudo curl --output-dir "/etc/yum.repos.d/" \
  --remote-name "https://copr.fedorainfracloud.org/coprs/avengemedia/dms/repo/fedora-$(rpm -E %fedora)/avengemedia-dms-fedora-$(rpm -E %fedora).repo"

dnf -y install quickshell dms greetd dms-greeter --allowerasing
