#!/bin/bash
set -ouex pipefail

echo "--- Applying Advanced Kernel & Boot Optimizations ---"

# Configurazioni Sysctl, ZRAM, Journald, BPF e Ananicy
# Tutte queste configurazioni sono state migrate nativamente su /system_files/
# garantendo un design architetturale OCI dichiarativo e pulito.

echo "--- Kernel Optimizations Applied ---"
