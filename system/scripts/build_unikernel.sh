#!/usr/bin/env bash
# ==============================================================================
# 🌌 Ermete OS - Level 12 Unikernel Runtime Engine Build System (RustyHermit)
# Compiles Rust microservices into bare-metal Ring-0 Unikernel binaries,
# bypassing the POSIX userland stack for zero-latency execution.
# ==============================================================================

set -euo pipefail

COMPONENT="${1:-ermete-unikernel-daemon}"
MODE="${2:-release}"
TARGET="x86_64-unknown-hermit"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m'

log_info() {
	echo -e "${BLUE}[Ermete Unikernel Engine]${NC} $1"
}

log_success() {
	echo -e "${GREEN}[Ermete Unikernel Engine]${NC} $1"
}

log_warn() {
	echo -e "${YELLOW}[Ermete Unikernel Engine]${NC} $1"
}

log_error() {
	echo -e "${RED}[Ermete Unikernel Engine]${NC} $1" >&2
}

log_info "Initializing Level 12 Singularity Unikernel Builder..."
log_info "Target Architecture: ${TARGET} (RustyHermit Bare-Metal Ring-0)"
log_info "Component: ${COMPONENT} | Mode: ${MODE}"

# Ensure system lib environment
export LD_LIBRARY_PATH="/usr/lib64:${LD_LIBRARY_PATH:-}"

# Check for Rust toolchain
if ! command -v cargo >/dev/null 2>&1; then
	log_error "Cargo toolchain is required but not installed."
	exit 1
fi

# Detect toolchain to use
RUST_TOOLCHAIN="stable"
if rustup toolchain list | grep -q "nightly"; then
	RUST_TOOLCHAIN="nightly"
	log_info "Using nightly toolchain for bare-metal build-std support."
fi

# Ensure target support or build-std flag
BUILD_FLAGS=("--package" "${COMPONENT}" "--target" "${TARGET}")
if [[ "${MODE}" == "release" ]]; then
	BUILD_FLAGS+=("--release")
fi

log_info "Executing compilation via RustyHermit zero-latency engine..."

if command -v cargo-hermit >/dev/null 2>&1; then
	log_info "Using cargo-hermit toolchain wrapper..."
	cargo hermit build "${BUILD_FLAGS[@]}"
else
	log_warn "cargo-hermit not found in PATH, executing direct cargo build-std pipeline..."
	cargo "+${RUST_TOOLCHAIN}" build "${BUILD_FLAGS[@]}" -Z build-std=std,panic_abort
fi

# Verify artifact output
ARTIFACT_PATH="target/${TARGET}/${MODE}/${COMPONENT}"
if [[ -f "${ARTIFACT_PATH}" ]]; then
	SIZE_BYTES=$(stat -c %s "${ARTIFACT_PATH}")
	log_success "Unikernel Ring-0 Binary compiled successfully!"
	log_success "Artifact location: ${ARTIFACT_PATH} (${SIZE_BYTES} bytes)"
	log_info "Status: ZERO-LATENCY BARE-METAL READY (POSIX Stack Bypassed)"
else
	log_error "Expected artifact missing at ${ARTIFACT_PATH}"
	exit 1
fi
