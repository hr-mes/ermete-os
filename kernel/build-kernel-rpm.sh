#!/bin/bash
set -eo pipefail

echo "========================================================="
echo " ERMETE OS CHIMERA KERNEL - RPM BUILDER"
echo "========================================================="

WORKSPACE_DIR="/workspace"
RPMBUILD_DIR="$WORKSPACE_DIR/kernel/rpmbuild_out"

echo ">>> Preparazione ambiente rpmbuild isolato..."
mkdir -p "$RPMBUILD_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

cd "$WORKSPACE_DIR/kernel"

echo ">>> Download del Kernel CachyOS (Sorgente Pura)..."
git clone --depth 1 https://github.com/CachyOS/linux.git cachyos-tree

echo ">>> Download del file di configurazione base CachyOS..."
[ ! -f cachyos-base.cfg ] && curl -sL "https://raw.githubusercontent.com/CachyOS/linux-cachyos/master/linux-cachyos-bore/config" -o cachyos-base.cfg || echo "Configurazione già presente in cache."

echo ">>> Creazione del file ermete-bedrock.cfg (Fusione configurazioni)..."
cat cachyos-base.cfg > "$RPMBUILD_DIR/SOURCES/ermete-bedrock.cfg"
cat ermete-bedrock.cfg >> "$RPMBUILD_DIR/SOURCES/ermete-bedrock.cfg"

echo ">>> Ottimizzazione: Evitiamo tarball, spec farà rsync in %prep"
# mkdir e rsync sono stati spostati nel %prep dello spec per supportare directory versionate di rpmbuild

echo ">>> Copia dei file di configurazione e Spec..."
cp ermete-kernel.spec "$RPMBUILD_DIR/SPECS/"

echo ">>> Rilevamento della stringa di rilascio del Kernel (uname -r)..."
KVER=$(awk '/^VERSION =/ {print $3}' cachyos-tree/Makefile)
KPATCH=$(awk '/^PATCHLEVEL =/ {print $3}' cachyos-tree/Makefile)
KSUB=$(awk '/^SUBLEVEL =/ {print $3}' cachyos-tree/Makefile)
KEXTRA=$(awk '/^EXTRAVERSION =/ {print $3}' cachyos-tree/Makefile)
if [ -z "$KSUB" ]; then
  KREL="${KVER}.${KPATCH}${KEXTRA}"
else
KREL="${KVER}.${KPATCH}.${KSUB}${KEXTRA}"
fi
# RPM version cannot contain hyphens, replace with tilde
KVERSION="${KREL//-/\~}"
echo "KREL rilevato: $KREL (KVERSION: $KVERSION)"

echo ">>> Avvio compilazione RPM tramite rpmbuild..."
rpmbuild --define "_topdir $RPMBUILD_DIR" --define "krel $KREL" --define "kernel_version $KVERSION" -ba "$RPMBUILD_DIR/SPECS/ermete-kernel.spec"

echo "========================================================="
echo " COMPILAZIONE COMPLETATA "
echo "========================================================="
echo "I pacchetti RPM si trovano in: $RPMBUILD_DIR/RPMS/x86_64/"

echo ">>> Statistiche CCACHE:"
ccache -s || true
