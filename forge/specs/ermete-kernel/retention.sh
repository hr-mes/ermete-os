#!/usr/bin/env bash
# Retention dei pacchetti OCI del kernel su ghcr (docs/architecture/doc_kernel_build.md,
# sezione 3, passo 8). Una versione (= un manifesto) resta se e' raggiungibile da una
# release conservata; tutto il resto se ne va.
#
# Release: un manifesto con un tag che non inizia per `sha256-` (l'NVR, `latest`). Ne
# restano le KEEP piu' recenti: tutte per kernel e devel, due per il debuginfo.
# Raggiungibile da una release con digest sha256:<hex>:
#   - i manifesti con tag `sha256-<hex>*`: l'indice dei referrer che cosign v3 aggiorna
#     sotto il tag di fallback `sha256-<hex>` (ghcr non ha l'API referrers: GET
#     /v2/<repo>/referrers/<digest> risponde 404) e i tag legacy `.sig`, `.att`, `.sbom`;
#   - i membri di quell'indice, cioe' i bundle Sigstore di firma e attestazioni, che sono
#     manifesti senza tag: la lista si legge dal registro, non dall'API dei pacchetti.
# Se ne vanno quindi le release oltre KEEP con i loro referrer, gli indici sostituiti da
# ogni `cosign sign|attest` successivo e i manifesti di un push ripetuto dello stesso tag.
# Il gate del workflow verifica la firma DOPO la retention: se questo modello smettesse
# di valere (ghcr con l'API referrers, cosign senza tag di fallback) il run fallirebbe.
#
# Uso: retention.sh [--dry-run]. Serve gh con read:packages e delete:packages (in CI il
# GITHUB_TOKEN con packages: write) e skopeo autenticato su ghcr (login di buildah).
set -euo pipefail

DRY=''
[[ ${1:-} == --dry-run ]] && DRY=1
OWNER=${GITHUB_REPOSITORY_OWNER:-hr-mes}

prune() { # prune PACKAGE KEEP
  local pkg=$1 keep=$2 api versions digest hex member
  local -A live=()
  api="/users/${OWNER}/packages/container/${pkg}/versions"
  versions=$(gh api --paginate "${api}?per_page=100" | jq -s 'add // []')

  while read -r digest; do
    live[$digest]=1
    hex=${digest#sha256:}
    while read -r member; do live[$member]=1; done < <(jq -r --arg hex "$hex" '
      .[] | select(.metadata.container.tags | any(startswith("sha256-" + $hex))) | .name' <<<"$versions")
    if jq -e --arg tag "sha256-${hex}" 'any(.[]; .metadata.container.tags | index($tag))' <<<"$versions" > /dev/null; then
      while read -r member; do live[$member]=1; done < <(
        skopeo inspect --raw "docker://ghcr.io/${OWNER}/${pkg}:sha256-${hex}" | jq -r '.manifests[].digest')
    fi
  done < <(jq -r --argjson keep "$keep" '
    [.[] | select(.metadata.container.tags | any(startswith("sha256-") | not))]
    | sort_by(.created_at) | reverse | .[:$keep][].name' <<<"$versions")

  echo "${pkg}: ${#live[@]} manifesti raggiungibili da una release conservata"
  while read -r id digest tags; do
    [[ ${live[$digest]:-} ]] && continue
    echo "${DRY:+[dry-run] }${pkg}: cancello ${id} ${digest} ${tags:-(senza tag)}"
    [[ $DRY ]] || gh api --method DELETE "${api}/${id}" > /dev/null
  done < <(jq -r '.[] | "\(.id) \(.name) \(.metadata.container.tags | join(","))"' <<<"$versions")
}

prune ermete-os-kernel 1000000
prune ermete-os-kernel-devel 1000000
prune ermete-os-kernel-debuginfo 2
prune ermete-os-nvidia 1000000
