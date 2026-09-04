#!/usr/bin/env bash
# Retention dei pacchetti OCI del kernel su ghcr (docs/architecture/doc_kernel_build.md,
# sezione 3, passo 8). Per ogni pacchetto restano le KEEP versioni con tag NVR piu'
# recenti: tutte per kernel e devel, due per il debuginfo. Se ne vanno le altre versioni
# con tag NVR, ogni versione senza tag (un manifesto sostituito da un nuovo push dello
# stesso tag: la provenance sta nello store di GitHub, non nel registro, quindi nessun
# referrer legittimo e' senza tag) e i referrer cosign (sha256-<hex>.sig, .att) di cio'
# che se ne va.
# Uso: retention.sh [--dry-run]. Serve gh autenticato con read:packages e delete:packages
# (in CI il GITHUB_TOKEN con packages: write).
set -euo pipefail

DRY=''
[[ ${1:-} == --dry-run ]] && DRY=1
OWNER=${GITHUB_REPOSITORY_OWNER:-hr-mes}

prune() { # prune PACKAGE KEEP
  local pkg=$1 keep=$2 api versions
  api="/users/${OWNER}/packages/container/${pkg}/versions"
  versions=$(gh api --paginate "${api}?per_page=100" | jq -s 'add // []')
  jq -r --argjson keep "$keep" '
    def nvr_tagged: select(.metadata.container.tags | any(startswith("sha256-") | not));
    def untagged: select(.metadata.container.tags | length == 0);
    ([.[] | nvr_tagged] | sort_by(.created_at) | reverse) as $tagged
    | [ $tagged[$keep:][], (.[] | untagged) ]
    | .[].name' <<<"$versions" \
  | while read -r digest; do
      hex=${digest#sha256:}
      jq -r --arg hex "$hex" '
        .[] | select(.name == "sha256:" + $hex or (.metadata.container.tags | any(startswith("sha256-" + $hex))))
        | "\(.id) \(.name) \(.metadata.container.tags | join(","))"' <<<"$versions" \
      | while read -r id name tags; do
          echo "${DRY:+[dry-run] }${pkg}: cancello ${id} ${name} ${tags:-(senza tag)}"
          [[ $DRY ]] || gh api --method DELETE "${api}/${id}" > /dev/null
        done
    done
}

prune ermete-os-kernel 1000000
prune ermete-os-kernel-devel 1000000
prune ermete-os-kernel-debuginfo 2
