#!/usr/bin/env bash
# L'NVR del kernel Ermete derivato dai pin: lo stesso che rpmbuild produce con
# `%buildid .ermete` (build.sh) e che publish usa come tag delle immagini OCI.
# Uso: nvr.sh [pins.env]
set -euo pipefail
# shellcheck source=pins.env
source "${1:-$(dirname "${BASH_SOURCE[0]}")/pins.env}"
rel=${FEDORA_KERNEL_NVR#*-}
echo "${FEDORA_KERNEL_NVR%%-*}-${rel%%.*}.ermete.${rel#*.}"
