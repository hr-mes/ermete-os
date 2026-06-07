#!/bin/bash
set -ouex pipefail

echo "--- Cleaning up image ---"

# Clean up dnf cache to reduce image size
# FIX: Sostituzione radicale dei comandi imperativi rm -rf che causavano la destrutturazione
# silente del bootc lint e paralizzavano l'algoritmo di relabeling di SELinux MAC.
dnf -y clean all
dnf5 -y clean all

# Rimozione esclusiva e puntuale dei file binari temporanei scaricati.
# La struttura di /var/lib/dnf resta inalterata e pronta per l'operatività del first-boot.
rm -rf /var/cache/dnf/*
rm -rf /var/cache/libdnf5/*
