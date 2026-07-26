#!/bin/bash
set -euo pipefail
# Bedrock Pure Bash Idempotency Checker
# Replaces python3 idempotency_checker.py with native system tools (find, sha256sum, skopeo)

PACKAGE=""
REGISTRY=""
OWNER=""
IMAGE_NAME=""

BASE_DIGEST=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --package) PACKAGE="$2"; shift 2 ;;
    --registry) REGISTRY="$2"; shift 2 ;;
    --owner) OWNER="$2"; shift 2 ;;
    --image-name) IMAGE_NAME="$2"; shift 2 ;;
    --base-digest) BASE_DIGEST="$2"; shift 2 ;;
    *) echo "Argomento sconosciuto: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$IMAGE_NAME" ]]; then
  IMAGE_NAME="ermete-os-forge-${PACKAGE}"
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
    find "$DIR" -type f -print0 | sort -z | xargs -0 sha256sum
    if [[ -f "config/rpmmacros" ]]; then
      echo -n "config/rpmmacros"
      cat "config/rpmmacros"
    fi
    if [[ -f "builder/Containerfile" ]]; then
      echo -n "builder/Containerfile"
      cat "builder/Containerfile"
    fi
    if [[ -f "builder/rpmfusion-custom.repo" ]]; then
      echo -n "builder/rpmfusion-custom.repo"
      cat "builder/rpmfusion-custom.repo"
    fi
    if [[ -f "config/packages.json" ]]; then
      echo -n "config/packages.json"
      cat "config/packages.json"
    fi
  } | sha256sum | awk '{print $1}')
else
  # Pacchetti upstream senza spec locale
  if command -v dnf >/dev/null 2>&1; then
    # Cerchiamo la versione effettiva nei repository abilitati usando cacheonly per evitare lock concorrenti
    UPSTREAM_VER=$(dnf repoquery --cacheonly --qf "%{VERSION}-%{RELEASE}\n" --arch x86_64,noarch "$PACKAGE" 2>/dev/null | sort -V | tail -n 1 || true)
  else
    UPSTREAM_VER=""
  fi
  
  # Invalidiamo la cache degli upstream (compilati da zero) ad ogni aggiornamento della Base Image
  # per prevenire desincronizzazione librerie (es. libx265 per ffmpeg).
  if [[ -z "${BASE_DIGEST:-}" ]]; then
    if command -v skopeo >/dev/null 2>&1; then
      BASE_DIGEST=$(skopeo inspect --no-tags "docker://ghcr.io/${OWNER}/ermete-base-nvidia:latest" 2>/dev/null | grep -oP '"Digest": "\K[^"]+' | head -n 1 || true)
    fi
  fi
  
  VERSION=${UPSTREAM_VER:-unknown}
  if [[ -n "$UPSTREAM_VER" ]]; then
    CONTENT_HASH=$(echo -n "${PACKAGE}-${UPSTREAM_VER}-${BASE_DIGEST}-v4" | sha256sum | awk '{print $1}')
  else
    CONTENT_HASH=$(echo -n "${PACKAGE}-${VERSION}-upstream-v4-${BASE_DIGEST}" | sha256sum | awk '{print $1}')
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
    sudo dnf install -y skopeo >&2 || dnf install -y skopeo >&2 || :
  fi
fi

CACHE_HIT="false"
if command -v skopeo >/dev/null 2>&1; then
  INSPECT_ARGS=("--no-tags")
  if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    INSPECT_ARGS+=("--creds" "${OWNER}:${GITHUB_TOKEN}")
  fi
  if skopeo inspect "${INSPECT_ARGS[@]}" "${IMAGE_URL_LOWER}" >/dev/null 2>&1; then
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
