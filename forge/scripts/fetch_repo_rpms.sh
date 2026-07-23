#!/bin/bash
set -euo pipefail

mkdir -p /github/home/repo \
         /github/home/repo-tier0 \
         /github/home/repo-tier1 \
         /github/home/repo-tier2 \
         /github/home/repo-tier3

export STORAGE_DRIVER=overlay
export BUILDAH_ISOLATION=chroot

TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

OWNER="${1:-patapem}"

# Fetch package lists dynamically from Single Source of Truth
readarray -t UPSTREAM_CORE < <(jq -r '.upstream_core[]' config/packages.json)
readarray -t UPSTREAM_DESKTOP < <(jq -r '.upstream_desktop[]' config/packages.json)
readarray -t UPSTREAM_MEDIA < <(jq -r '.upstream_media[]' config/packages.json)
readarray -t UPSTREAM_CLI < <(jq -r '.upstream_cli[]' config/packages.json)

# Define per-Tier micro-container images
TIER0_IMAGES=(
  "ermete-kernel-source"
  "ermete-forge-nvidia"
  "ermete-forge-base-config"
  "ermete-forge-selinux"
  "ermete-forge-nix-support"
  "ermete-forge-recovery"
  "ermete-forge-secure-boot"
)
for pkg in "${UPSTREAM_CORE[@]}" "${UPSTREAM_MEDIA[@]}"; do
  [[ -n "$pkg" ]] && TIER0_IMAGES+=("ermete-forge-rolling-$pkg")
done

TIER1_IMAGES=(
  "ermete-forge-starship"
  "ermete-forge-bat"
  "ermete-forge-ananicy"
  "ermete-forge-cliphist"
  "ermete-forge-ide-bootstrap"
  "ermete-forge-system-tweaks"
)
for pkg in "${UPSTREAM_DESKTOP[@]}" "${UPSTREAM_CLI[@]}"; do
  [[ -n "$pkg" ]] && TIER1_IMAGES+=("ermete-forge-rolling-$pkg")
done

TIER2_IMAGES=(
  "ermete-forge-bibata"
  "ermete-forge-matugen"
  "ermete-forge-dart-sass"
)

TIER3_IMAGES=(
  "ermete-forge-shell-rs"
  "ermete-forge-settings-rs"
  "ermete-forge-daemon-rs"
  "ermete-forge-store-rs"
  "ermete-forge-doctor"
  "ermete-forge-system-services"
  "ermete-forge-desktop-ui"
  "ermete-forge-system-config"
  "ermete-forge-backup"
  "ermete-forge-gatekeeper-rs"
  "ermete-forge-telemetry-rs"
  "ermete-forge-cloud-rs"
  "ermete-forge-mdm-rs"
  "ermete-forge-updater-rs"
  "ermete-forge-xdg-desktop-portal-ermete"
  "ermete-forge-lvfs-rs"
  "ermete-forge-ui-agent"
)

declare -A OLD_DIGESTS
declare -A NEW_DIGESTS

pull_and_extract() {
  local img="$1"
  local target_dir="$2"
  local IMAGE_LOWER=$(echo "ghcr.io/$OWNER/$img:latest" | tr '[:upper:]' '[:lower:]')
  
  local old_digest="${OLD_DIGESTS[$img]:-}"
  local new_digest=$(skopeo inspect --config "docker://$IMAGE_LOWER" 2>/dev/null | jq -r '.config.Labels["org.opencontainers.image.revision"] // .config.Labels["tier.content.sha256"] // ""' || true)
  if [ -z "$new_digest" ]; then
    new_digest=$(skopeo inspect "docker://$IMAGE_LOWER" 2>/dev/null | jq -r '.Digest' || true)
  fi

  if [ -n "$old_digest" ] && [ -n "$new_digest" ] && [ "$old_digest" = "$new_digest" ]; then
    echo "    [CACHE HIT] $img hasn't changed. Skipping pull."
    echo "$new_digest" > "$TMP_DIR/digest_$img"
    return 0
  fi
  
  echo "    [CACHE MISS] Pulling $img (old: $old_digest, new: $new_digest)"
  echo "$new_digest" > "$TMP_DIR/digest_$img"

  local ctr
  ctr=$(buildah from "$IMAGE_LOWER" || true)
  if [ -n "$ctr" ]; then
    local mnt
    mnt=$(buildah mount "$ctr")
    
    # Prune old versions of the same package to avoid conflicting requests downstream
    for new_rpm in "$mnt"/*.rpm; do
      if [ -f "$new_rpm" ]; then
        local pkg_name
        pkg_name=$(rpm -qp --queryformat '%{NAME}' "$new_rpm" 2>/dev/null || true)
        if [ -n "$pkg_name" ]; then
          (
            flock 200
            rm -f "$target_dir/${pkg_name}"-[0-9]*.rpm 2>/dev/null || true
          ) 200>"$target_dir/.lock"
        fi
      fi
    done

    (
      flock 200
      cp -a "$mnt"/*.rpm "$target_dir/" 2>/dev/null || true
    ) 200>"$target_dir/.lock"
    buildah umount "$ctr"
    buildah rm "$ctr"
  else
    echo "    [!] Immagine non trovata o scaricamento fallito per $img"
  fi
}

echo "=== Restoring Aggregate Tier Repos (Caching) ==="
for tier in tier0 tier1 tier2 tier3; do
  img="ermete-forge-${tier}-repo:latest"
  IMAGE_LOWER=$(echo "ghcr.io/$OWNER/$img" | tr '[:upper:]' '[:lower:]')
  echo "    Pulling previous $IMAGE_LOWER..."
  ctr=$(buildah from "$IMAGE_LOWER" 2>/dev/null || true)
  if [ -n "$ctr" ]; then
    mnt=$(buildah mount "$ctr")
    cp -a "$mnt"/*.rpm "/github/home/repo-${tier}/" 2>/dev/null || true
    cp -a "$mnt"/*.rpm "/github/home/repo/" 2>/dev/null || true
    if [ -f "$mnt/manifest.json" ]; then
      cp -a "$mnt/manifest.json" "/github/home/repo-${tier}/" 2>/dev/null || true
    fi
    buildah umount "$ctr"
    buildah rm "$ctr"
  fi
  
  # Load old digests
  if [ -f "/github/home/repo-${tier}/manifest.json" ]; then
    while read -r k v; do
      OLD_DIGESTS["$k"]="$v"
    done < <(jq -r 'to_entries[] | "\(.key) \(.value)"' "/github/home/repo-${tier}/manifest.json")
  fi
done

echo "=== Extracting Tier 0 RPMs ==="
for img in "${TIER0_IMAGES[@]}"; do
  pull_and_extract "$img" "/github/home/repo-tier0" &
done

echo "=== Extracting Tier 1 RPMs ==="
for img in "${TIER1_IMAGES[@]}"; do
  pull_and_extract "$img" "/github/home/repo-tier1" &
done

echo "=== Extracting Tier 2 RPMs ==="
for img in "${TIER2_IMAGES[@]}"; do
  pull_and_extract "$img" "/github/home/repo-tier2" &
done

echo "=== Extracting Tier 3 RPMs ==="
for img in "${TIER3_IMAGES[@]}"; do
  pull_and_extract "$img" "/github/home/repo-tier3" &
done

for pid in $(jobs -p); do
  wait $pid || { echo "FATAL: Un job in parallelo è fallito"; exit 1; }
done

echo "=== Syncing tiered RPMs to aggregate repo ==="
cp -a /github/home/repo-tier0/*.rpm /github/home/repo/ 2>/dev/null || true
cp -a /github/home/repo-tier1/*.rpm /github/home/repo/ 2>/dev/null || true
cp -a /github/home/repo-tier2/*.rpm /github/home/repo/ 2>/dev/null || true
cp -a /github/home/repo-tier3/*.rpm /github/home/repo/ 2>/dev/null || true

for img in "${TIER0_IMAGES[@]}" "${TIER1_IMAGES[@]}" "${TIER2_IMAGES[@]}" "${TIER3_IMAGES[@]}"; do
  if [ -f "$TMP_DIR/digest_$img" ]; then
    NEW_DIGESTS["$img"]=$(cat "$TMP_DIR/digest_$img")
  fi
done

echo "=== Saving New Manifests ==="
for tier in tier0 tier1 tier2 tier3; do
  # We construct the manifest for the tier
  echo "{}" > "/github/home/repo-${tier}/manifest.json"
  declare -n TIER_ARRAY="TIER${tier#tier}_IMAGES"
  for img in "${TIER_ARRAY[@]}"; do
    digest="${NEW_DIGESTS[$img]:-}"
    if [ -n "$digest" ]; then
      jq --arg k "$img" --arg v "$digest" '.[$k] = $v' "/github/home/repo-${tier}/manifest.json" > tmp.json && mv tmp.json "/github/home/repo-${tier}/manifest.json"
    fi
  done
  cp "/github/home/repo-${tier}/manifest.json" "/github/home/repo/manifest_${tier}.json" 2>/dev/null || true
done

echo "--- Extracted RPMs Summary ---"
echo "Tier 0 count: $(ls -1 /github/home/repo-tier0/*.rpm 2>/dev/null | wc -l || echo 0)"
echo "Tier 1 count: $(ls -1 /github/home/repo-tier1/*.rpm 2>/dev/null | wc -l || echo 0)"
echo "Tier 2 count: $(ls -1 /github/home/repo-tier2/*.rpm 2>/dev/null | wc -l || echo 0)"
echo "Tier 3 count: $(ls -1 /github/home/repo-tier3/*.rpm 2>/dev/null | wc -l || echo 0)"
echo "Total repo count: $(ls -1 /github/home/repo/*.rpm 2>/dev/null | wc -l || echo 0)"
