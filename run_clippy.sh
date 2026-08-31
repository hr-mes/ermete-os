#!/bin/bash
podman run --rm -v "$PWD":/workspace -w /workspace ghcr.io/hr-mes/ermete-os-builder:latest bash -c "rm -f Cargo.lock && cargo clippy --workspace --exclude ebpf-core --exclude ermete-sysmon-ebpf --all-targets --all-features -- -A clippy::undocumented_unsafe_blocks -A clippy::multiple_unsafe_ops_per_block -D warnings -A dead_code -A unused_variables"
