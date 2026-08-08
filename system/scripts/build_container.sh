#!/bin/bash
set -euo pipefail

TARGET_IMAGE="${1:-ermete-os-system}"
TAG="${2:-latest}"

BUILD_ARGS=()
if [[ -z "$(git status -s)" ]]; then
    BUILD_ARGS+=("--build-arg" "SHA_HEAD_SHORT=$(git rev-parse --short HEAD)")
fi

podman build "${BUILD_ARGS[@]}" --pull=newer --tag "${TARGET_IMAGE}:${TAG}" .
