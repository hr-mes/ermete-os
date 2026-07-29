#!/bin/bash
set -euo pipefail

REGISTRY="ghcr.io"
OWNER="${GITHUB_REPOSITORY_OWNER:-hr-mes}"

if ! command -v skopeo >/dev/null 2>&1 || ! command -v jq >/dev/null 2>&1; then
  if command -v dnf >/dev/null 2>&1; then
    dnf install -y skopeo jq
  else
    sudo apt-get update && sudo apt-get install -y skopeo jq
  fi
fi

# Fetch arrays dynamically from Single Source of Truth
readarray -t CUSTOM_PKGS < <(jq -r '.custom_packages[]' config/packages.json)
readarray -t UPSTREAM_CORE < <(jq -r '.upstream_core[]' config/packages.json)
readarray -t UPSTREAM_DESKTOP < <(jq -r '.upstream_desktop[]' config/packages.json)
readarray -t UPSTREAM_MEDIA < <(jq -r '.upstream_media[]' config/packages.json)
readarray -t UPSTREAM_CLI < <(jq -r '.upstream_cli[]' config/packages.json)

# Create a directory for DNF cache to share across containers
mkdir -p .dnf-cache

# Pre-fetch Base Digest to avoid 60 unnecessary skopeo network calls (saves ~5 minutes!)
echo "Fetching base image digest..." >&2
BASE_DIGEST=$(skopeo inspect --no-tags "docker://ghcr.io/${OWNER}/ermete-base-nvidia:latest" 2>/dev/null | jq -r '.Digest' || echo "")
echo "Base digest: $BASE_DIGEST" >&2

process_array() {
  local prefix=$1
  shift
  local pkgs=("$@")
  
  local active_pkgs=()
  local script_args=()
  
  for pkg in "${pkgs[@]}"; do
    pkg="${pkg//,/}"
    [[ -z "$pkg" ]] && continue
    script_args+=("$pkg" "ermete-os-forge-${prefix}${pkg}")
  done
  
  if [[ ${#script_args[@]} -eq 0 ]]; then
    jq -c -n '$ARGS.positional' --args "${active_pkgs[@]}"
    return
  fi
  
  # Run podman ONCE for all packages in this array, running checks in PARALLEL
  local out
  out=$(podman run -i --rm --security-opt label=disable --security-opt seccomp=unconfined -e GITHUB_TOKEN="${GITHUB_TOKEN:-}" -v "$(pwd):/workspace" -v "$(pwd)/.dnf-cache:/var/cache/dnf" -w /workspace ghcr.io/${OWNER}/ermete-os-builder:latest bash -c '
    BASE_DIGEST=$1; shift
    REGISTRY=$1; shift
    OWNER=$1; shift
    
    # Pre-populate DNF cache sequentially to prevent locking issues during parallel queries
    dnf makecache --refresh 2>/dev/null || true
    
    cat << \EOF2 > /tmp/run_check.sh
BASE_DIGEST=$1
REGISTRY=$2
OWNER=$3
pkg=$4
img_name=$5
timeout 20s bash scripts/check_idempotency.sh --package "$pkg" --registry "$REGISTRY" --owner "$OWNER" --image-name "$img_name" --base-digest "$BASE_DIGEST" > "/tmp/${pkg}_res" 2>/dev/null
EOF2

    echo "$@" | xargs -n 2 -P 5 bash /tmp/run_check.sh "$BASE_DIGEST" "$REGISTRY" "$OWNER"
    
    for f in /tmp/*_res; do
      [ -e "$f" ] || continue
      res=$(cat "$f")
      pkg=$(basename "$f" _res)
      if echo "$res" | grep -q "CACHE_HIT=false"; then
        echo "RESULT:$pkg:MISS"
      else
        echo "RESULT:$pkg:HIT"
      fi
    done
  ' -- "$BASE_DIGEST" "$REGISTRY" "$OWNER" "${script_args[@]}" 2>/dev/null)
  
  for pkg in "${pkgs[@]}"; do
    pkg="${pkg//,/}"
    [[ -z "$pkg" ]] && continue
    
    if echo "$out" | grep -q "RESULT:$pkg:MISS"; then
      active_pkgs+=("$pkg")
      echo "  -> MISS (will build: $pkg)" >&2
    elif echo "$out" | grep -q "RESULT:$pkg:HIT"; then
      echo "  -> HIT (skip: $pkg)" >&2
    else
      # Fallback in case of errors
      active_pkgs+=("$pkg")
      echo "  -> ERROR/MISS (will build: $pkg)" >&2
    fi
  done
  
  jq -c -n '$ARGS.positional' --args "${active_pkgs[@]}"
}

echo "Evaluating custom_packages..." >&2
J_CUSTOM=$(process_array "" "${CUSTOM_PKGS[@]}")

echo "Evaluating upstream_core..." >&2
J_U_CORE=$(process_array "rolling-" "${UPSTREAM_CORE[@]}")

echo "Evaluating upstream_desktop..." >&2
J_U_DESK=$(process_array "rolling-" "${UPSTREAM_DESKTOP[@]}")

echo "Evaluating upstream_media..." >&2
J_U_MEDIA=$(process_array "rolling-" "${UPSTREAM_MEDIA[@]}")

echo "Evaluating upstream_cli..." >&2
J_U_CLI=$(process_array "rolling-" "${UPSTREAM_CLI[@]}")

# Combine all upstream packages needing rebuild into a single array
J_UPSTREAM=$(jq -c -s 'add' <(echo "$J_U_CORE") <(echo "$J_U_DESK") <(echo "$J_U_MEDIA") <(echo "$J_U_CLI"))

# Determine if there are any changes across all packages
if [[ "$J_CUSTOM" != "[]" || "$J_UPSTREAM" != "[]" ]]; then
  HAS_CHANGES="true"
else
  HAS_CHANGES="false"
fi

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "custom_packages=${J_CUSTOM}" >> "$GITHUB_OUTPUT"
  echo "upstream_packages=${J_UPSTREAM}" >> "$GITHUB_OUTPUT"
  echo "upstream_core=${J_U_CORE}" >> "$GITHUB_OUTPUT"
  echo "upstream_desktop=${J_U_DESK}" >> "$GITHUB_OUTPUT"
  echo "upstream_media=${J_U_MEDIA}" >> "$GITHUB_OUTPUT"
  echo "upstream_cli=${J_U_CLI}" >> "$GITHUB_OUTPUT"
  echo "has_changes=${HAS_CHANGES}" >> "$GITHUB_OUTPUT"
fi

echo "JSON Outputs:"
echo "custom_packages=${J_CUSTOM}"
echo "upstream_packages=${J_UPSTREAM}"
echo "upstream_core=${J_U_CORE}"
echo "upstream_desktop=${J_U_DESK}"
echo "upstream_media=${J_U_MEDIA}"
echo "upstream_cli=${J_U_CLI}"
echo "has_changes=${HAS_CHANGES}"
