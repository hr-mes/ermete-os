#!/bin/bash
set -ouex pipefail

echo "--- Adding third-party repositories and codecs ---"

# Fully-featured ffmpeg with nonfree components from rpm fusion (System codecs)
# GUI apps (come OBS) rimosse e demandate a Flatpak per purezza dell'OS.
dnf -y install --setopt=install_weak_deps=False https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm
dnf -y install --setopt=install_weak_deps=False ffmpeg x264-libs libva-utils --allowerasing

