#!/bin/bash
set -euo pipefail
# Bedrock Pure Bash Idempotency Checker
# Replaces python3 idempotency_checker.py with native system tools (find, sha256sum, skopeo)

PACKAGE=""
REGISTRY=""
OWNER=""
IMAGE_NAME=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --package) PACKAGE="$2"; shift 2 ;;
    --registry) REGISTRY="$2"; shift 2 ;;
    --owner) OWNER="$2"; shift 2 ;;
    --image-name) IMAGE_NAME="$2"; shift 2 ;;
    *) echo "Argomento sconosciuto: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$IMAGE_NAME" ]]; then
  IMAGE_NAME="ermete-forge-${PACKAGE}"
fi

# Determina directory o seed per il calcolo dell'hash
if [[ "$PACKAGE" == "builder" ]]; then
  DIR="builder"
elif [[ -d "specs/ermete-${PACKAGE}" ]]; then
  DIR="specs/ermete-${PACKAGE}"
else
  DIR=""
fi

if [[ -n "$DIR" && -d "$DIR" ]]; then
  # Hash SHA-256 deterministico dei path relativi e dei contenuti
  CONTENT_HASH=$({
    find "$DIR" -type f | sort | while read -r f; do
      echo -n "${f#$DIR/}"
      cat "$f"
    done
    if [[ -f "config/rpmmacros" ]]; then
      echo -n "config/rpmmacros"
      cat "config/rpmmacros"
    fi
  } | sha256sum | awk '{print $1}')
else
  # Pacchetti upstream senza spec locale
  if command -v dnf >/dev/null 2>&1; then
    # Cerchiamo la versione effettiva nei repository abilitati
    UPSTREAM_VER=$(dnf repoquery --qf "%{VERSION}-%{RELEASE}" --arch x86_64,noarch "$PACKAGE" 2>/dev/null | sort -V | tail -n 1 || true)
  else
    UPSTREAM_VER=""
  fi
  
  VERSION=${UPSTREAM_VER:-unknown}
  if [[ -n "$UPSTREAM_VER" ]]; then
    CONTENT_HASH=$(echo -n "${PACKAGE}-${UPSTREAM_VER}" | sha256sum | awk '{print $1}')
  else
    CONTENT_HASH=$(echo -n "${PACKAGE}-${VERSION}-upstream-v1" | sha256sum | awk '{print $1}')
  fi
fi

echo ">>> Content Hash calcolato per ${PACKAGE}: ${CONTENT_HASH}" >&2

# Costruisce URL immagine GHCR
IMAGE_URL="docker://${REGISTRY}/${OWNER}/${IMAGE_NAME}:${CONTENT_HASH}"
IMAGE_URL_LOWER=$(echo "$IMAGE_URL" | tr '[:upper:]' '[:lower:]')

echo ">>> Verifica esistenza su GHCR: ${IMAGE_URL_LOWER}..." >&2

# Verifica con skopeo (se skopeo non è installato, tenta di installarlo o usa fallback)
if ! command -v skopeo >/dev/null 2>&1; then
  if command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y skopeo >&2 || dnf install -y skopeo >&2 || true
  fi
fi

CACHE_HIT="false"
if command -v skopeo >/dev/null 2>&1; then
  INSPECT_CMD="skopeo inspect --no-tags"
  if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    INSPECT_CMD="skopeo inspect --creds ${OWNER}:${GITHUB_TOKEN} --no-tags"
  fi
  if $INSPECT_CMD "${IMAGE_URL_LOWER}" >/dev/null 2>&1; then
    CACHE_HIT="true"
  fi
fi

echo "CACHE_HIT=${CACHE_HIT}"
echo "CONTENT_HASH=${CONTENT_HASH}"
if [[ "$CACHE_HIT" == "true" ]]; then
  echo ">>> Cache Hit! L'immagine esiste già su GHCR." >&2
else
  echo ">>> Cache Miss. Procedo con la build." >&2
fi
