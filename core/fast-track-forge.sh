#!/bin/bash
set -euo pipefail

PACKAGE=$1
if [ -z "$PACKAGE" ]; then
    echo "Uso: $0 <nome-pacchetto-forgia> (es. gatekeeper-rs, shell-rs)"
    exit 1
fi

echo "🚀 Fast-Tracking $PACKAGE dalla Forgia al Sistema Live..."

WORKDIR=$(mktemp -d)
trap 'rm -rf "$WORKDIR"' EXIT
cd "$WORKDIR"

echo "📥 Scaricamento del micro-container OCI ghcr.io/patapem/ermete-os-forge-$PACKAGE:latest..."
skopeo copy docker://ghcr.io/patapem/ermete-os-forge-$PACKAGE:latest dir:.

echo "📦 Estrazione RPM dall'immagine..."
mkdir -p extract
for file in *; do
  if [ -f "$file" ] && file "$file" | grep -qi "gzip compressed data"; then
    tar -xzf "$file" -C extract/
  fi
done

RPM_FILE=$(find extract/ -name "*.rpm" -print -quit || true)

if [ -z "$RPM_FILE" ]; then
    echo "❌ Nessun RPM trovato nel micro-container! Assicurati che la Forgia abbia finito di compilarlo."
    exit 1
fi

echo "⚡ Integrazione nativa OSTree in corso su $RPM_FILE..."
# Se il pacchetto è già installato nel base system, usiamo override replace. Altrimenti lo installiamo come layer addizionale.
if rpm -q "ermete-$PACKAGE" >/dev/null 2>&1; then
    echo "🔄 Il pacchetto esiste già. Eseguo l'override replace live..."
    sudo rpm-ostree override replace "$RPM_FILE" --apply-live --allow-replacement
else
    echo "➕ Pacchetto nuovo. Eseguo l'install live..."
    sudo rpm-ostree install "$RPM_FILE" --apply-live
fi

echo "✅ Fast-Track Completato! Il pacchetto è fuso nell'OSTree e persisterà ai riavvii."
echo "💡 Nota: Quando l'immagine base di Ermete OS finirà la sua build e farai un normale upgrade, OSTree assorbirà questo override automaticamente."
