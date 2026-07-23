#!/bin/bash
set -euo pipefail

REGISTRY="ghcr.io"
OWNER="${GITHUB_REPOSITORY_OWNER:-patapem}"

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

process_array() {
  local prefix=$1
  shift
  local pkgs=("$@")
  
  local active_pkgs=()
  
  for pkg in "${pkgs[@]}"; do
    pkg="${pkg//,/}"
    [[ -z "$pkg" ]] && continue
    
    local image_name="ermete-os-forge-${prefix}${pkg}"
    local out
    out=$(bash scripts/check_idempotency.sh --package "$pkg" --registry "$REGISTRY" --owner "$OWNER" --image-name "$image_name" 2>/dev/null)
    
    if echo "$out" | grep -q "CACHE_HIT=false"; then
      active_pkgs+=("$pkg")
      echo "  -> MISS (will build: $pkg)" >&2
    else
      echo "  -> HIT (skip: $pkg)" >&2
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

# Determine if there are any changes across all packages
HAS_CHANGES="false"
if [[ "$J_CUSTOM" != "[]" || "$J_U_CORE" != "[]" || "$J_U_DESK" != "[]" || "$J_U_MEDIA" != "[]" || "$J_U_CLI" != "[]" ]]; then
  HAS_CHANGES="true"
fi

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "custom_packages=${J_CUSTOM}" >> "$GITHUB_OUTPUT"
  echo "upstream_core=${J_U_CORE}" >> "$GITHUB_OUTPUT"
  echo "upstream_desktop=${J_U_DESK}" >> "$GITHUB_OUTPUT"
  echo "upstream_media=${J_U_MEDIA}" >> "$GITHUB_OUTPUT"
  echo "upstream_cli=${J_U_CLI}" >> "$GITHUB_OUTPUT"
  echo "has_changes=${HAS_CHANGES}" >> "$GITHUB_OUTPUT"
fi

echo "JSON Outputs:"
echo "custom_packages=${J_CUSTOM}"
echo "upstream_core=${J_U_CORE}"
echo "upstream_desktop=${J_U_DESK}"
echo "upstream_media=${J_U_MEDIA}"
echo "upstream_cli=${J_U_CLI}"
echo "has_changes=${HAS_CHANGES}"
