#!/bin/bash
set -eo pipefail

echo "========================================================="
echo " ERMETE OS CHIMERA KERNEL - RPM BUILDER"
echo "========================================================="

WORKSPACE_DIR="/workspace"
RPMBUILD_DIR="$WORKSPACE_DIR/rpmbuild_out"

echo ">>> Preparazione ambiente rpmbuild isolato..."
mkdir -p "$RPMBUILD_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

cd "$WORKSPACE_DIR"

echo ">>> Download del Kernel CachyOS (Sorgente Pura)..."
git clone --depth 1 https://github.com/CachyOS/linux.git cachyos-tree

echo ">>> Download del file di configurazione base CachyOS..."
curl -sL "https://raw.githubusercontent.com/CachyOS/linux-cachyos/master/linux-cachyos-bore/config" -o cachyos-base.cfg

echo ">>> Creazione del file ermete-bedrock.cfg (Fusione configurazioni)..."
cat cachyos-base.cfg > "$RPMBUILD_DIR/SOURCES/ermete-bedrock.cfg"
cat ermete-bedrock.cfg >> "$RPMBUILD_DIR/SOURCES/ermete-bedrock.cfg"

echo ">>> Generazione dell'archivio sorgente (linux-cachyos.tar.gz)..."
# Ignoriamo la cartella .git per ridurre il peso dell'archivio
tar --exclude='.git' -czf "$RPMBUILD_DIR/SOURCES/linux-cachyos.tar.gz" -C cachyos-tree .

echo ">>> Copia dei file di configurazione e Spec..."
cp ermete-kernel.spec "$RPMBUILD_DIR/SPECS/"

echo ">>> Avvio compilazione RPM tramite rpmbuild..."
rpmbuild --define "_topdir $RPMBUILD_DIR" -ba "$RPMBUILD_DIR/SPECS/ermete-kernel.spec"

echo "========================================================="
echo " COMPILAZIONE COMPLETATA "
echo "========================================================="
echo "I pacchetti RPM si trovano in: $RPMBUILD_DIR/RPMS/x86_64/"
