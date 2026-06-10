#!/bin/bash
set -ouex pipefail

echo "--- Adding third-party repositories and codecs ---"

# OBS and fully-featured ffmpeg with nonfree components from rpm fusion
dnf -y install --setopt=install_weak_deps=False https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm
dnf -y install --setopt=install_weak_deps=False ffmpeg x264-libs obs-studio obs-studio-plugin-x264 libva-utils --allowerasing

# Nautilus open any terminal extension
curl -Lo /etc/yum.repos.d/nautilus-open-any-terminal.repo \
  https://copr.fedorainfracloud.org/coprs/monkeygold/nautilus-open-any-terminal/repo/fedora-$(rpm -E %fedora)/monkeygold-nautilus-open-any-terminal-fedora-$(rpm -E %fedora).repo
