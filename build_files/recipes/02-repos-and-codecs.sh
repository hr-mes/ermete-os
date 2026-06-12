#!/bin/bash
set -ouex pipefail

echo "--- Adding third-party repositories and codecs ---"

# Fully-featured ffmpeg with nonfree components from rpm fusion (System codecs)
# GUI apps (come OBS) rimosse e demandate a Flatpak per purezza dell'OS.
# I repository RPMFusion sono già ereditati nativamente e verificati (Zero-Trust) dal Layer 0.
dnf -y install --setopt=install_weak_deps=False ffmpeg x264-libs libva-utils --allowerasing

