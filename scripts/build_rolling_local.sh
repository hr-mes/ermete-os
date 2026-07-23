#!/bin/bash
set -euo pipefail

if [ -z "$1" ]; then
    echo "Uso: $0 <nome-pacchetto>"
    echo "Esempio: $0 niri"
    exit 1
fi

PACKAGE=$1

echo "========================================"
echo "=== INIZIALIZZAZIONE AMBIENTE BEDROCK =="
echo "========================================"
sudo dnf install -y rpm-build dnf-plugins-core rpmdevtools
rm -rf ~/rpmbuild
cp config/rpmmacros ~/.rpmmacros
rpmdev-setuptree

echo "========================================"
echo "=== PREPARAZIONE REPOSITORIES (RPMFusion) ==="
echo "========================================"
sudo dnf install -y https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-43.noarch.rpm https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-43.noarch.rpm || true

echo "========================================"
echo "=== DOWNLOAD SORGENTI ==="
echo "========================================"
cd ~/rpmbuild/SRPMS
dnf download --source $PACKAGE

echo "========================================"
echo "=== INSTALLAZIONE DIPENDENZE E FIX ==="
echo "========================================"
sudo dnf builddep -y *.src.rpm



echo "========================================"
echo "=== ESTRAZIONE E INIEZIONE PONYTAIL ==="
echo "========================================"
rpm -ivh *.src.rpm

for spec in ~/rpmbuild/SPECS/*.spec; do
  if ! grep -q "debug_package %{nil}" "$spec"; then
    awk '/^Name:/ { print "%global debug_package %{nil}"; print $0; next } 1' "$spec" > "$spec.tmp" && mv "$spec.tmp" "$spec"
  fi
done

echo "========================================"
echo "=== COMPILAZIONE ESTREMA (ROLLING) ==="
echo "========================================"
rpmbuild -bb --nocheck ~/rpmbuild/SPECS/*.spec

echo "=================================================="
echo "🎯 PACCHETTO ROLLING '$PACKAGE' COMPILATO CON SUCCESSO! 🎯"
echo "I file RPM generati si trovano in ~/rpmbuild/RPMS/"
find ~/rpmbuild/RPMS -name "*.rpm"

# Esportazione sulla macchina Host (se /work è montato)
if [ -d "/work" ]; then
    mkdir -p /work/output/$PACKAGE
    cp ~/rpmbuild/RPMS/*/*.rpm /work/output/$PACKAGE/
    echo "RPMs esportati in /work/output/$PACKAGE/"
fi
echo "=================================================="
