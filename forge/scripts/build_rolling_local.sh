#!/bin/bash
set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Uso: $0 <nome-pacchetto>"
    echo "Esempio: $0 niri"
    exit 1
fi

PACKAGE="${1:-}"

echo "========================================"
echo "=== INIZIALIZZAZIONE AMBIENTE BEDROCK =="
echo "========================================"
sudo dnf install -y rpm-build dnf-plugins-core rpmdevtools
RPMBUILD_DIR=$(mktemp -d)
RPMMACROS_BAK=$(mktemp)
if [ -f ~/.rpmmacros ]; then
    cp ~/.rpmmacros "$RPMMACROS_BAK"
    trap 'rm -rf "$RPMBUILD_DIR"; cp "$RPMMACROS_BAK" ~/.rpmmacros; rm -f "$RPMMACROS_BAK"' EXIT
else
    trap 'rm -rf "$RPMBUILD_DIR"; rm -f ~/.rpmmacros; rm -f "$RPMMACROS_BAK"' EXIT
fi

echo "%_topdir $RPMBUILD_DIR" > ~/.rpmmacros
cat "$(dirname "$0")/../config/rpmmacros" >> ~/.rpmmacros
mkdir -p "$RPMBUILD_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

echo "========================================"
echo "=== PREPARAZIONE REPOSITORIES (RPMFusion) ==="
echo "========================================"
sudo dnf install -y https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-43.noarch.rpm https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-43.noarch.rpm || true


SPEC_DIR="$(realpath "$(dirname "$0")/../specs")"
CUSTOM_DIR="$SPEC_DIR/ermete-$PACKAGE"
if [ ! -d "$CUSTOM_DIR" ]; then
    CUSTOM_DIR="$SPEC_DIR/$PACKAGE"
fi

if [ -d "$CUSTOM_DIR" ]; then
    echo "========================================"
    echo "=== [ZERO-TRUST] PACCHETTO LOCALE ==="
    echo "========================================"
    cp -a "$CUSTOM_DIR"/* "$RPMBUILD_DIR"/SOURCES/
    mv "$RPMBUILD_DIR"/SOURCES/*.spec "$RPMBUILD_DIR"/SPECS/ 2>/dev/null || true
    echo "========================================"
    echo "=== INSTALLAZIONE DIPENDENZE E FIX ==="
    echo "========================================"
    sudo dnf builddep -y "$RPMBUILD_DIR"/SPECS/*.spec || true
else
    echo "========================================"
    echo "=== DOWNLOAD SORGENTI UPSTREAM ==="
    echo "========================================"
    cd "$RPMBUILD_DIR"/SRPMS
    dnf download --source "$PACKAGE"
    echo "========================================"
    echo "=== INSTALLAZIONE DIPENDENZE E FIX ==="
    echo "========================================"
    sudo dnf builddep -y *.src.rpm
    echo "========================================"
    echo "=== ESTRAZIONE E INIEZIONE PONYTAIL ==="
    echo "========================================"
    rpm -ivh *.src.rpm
fi


for spec in "$RPMBUILD_DIR"/SPECS/*.spec; do
  if ! grep -q "debug_package %{nil}" "$spec"; then
    awk '/^Name:/ { print "%global debug_package %{nil}"; print $0; next } 1' "$spec" > "$spec.tmp" && mv "$spec.tmp" "$spec"
  fi
done

echo "========================================"
echo "=== COMPILAZIONE ESTREMA (ROLLING) ==="
echo "========================================"
rpmbuild -bb --nocheck "$RPMBUILD_DIR"/SPECS/*.spec

echo "=================================================="
echo "🎯 PACCHETTO ROLLING '$PACKAGE' COMPILATO CON SUCCESSO! 🎯"
echo "I file RPM generati si trovano in \"$RPMBUILD_DIR\"/RPMS/"
find "$RPMBUILD_DIR"/RPMS -name "*.rpm"

# Esportazione sulla macchina Host (se /work è montato)
if [ -d "/work" ]; then
    mkdir -p "/work/output/$PACKAGE"
    cp "$RPMBUILD_DIR"/RPMS/*/*.rpm "/work/output/$PACKAGE/"
    echo "RPMs esportati in /work/output/$PACKAGE/"
fi
echo "=================================================="
