#!/bin/bash
set -e
cat << 'EOF' > Dockerfile.clippy
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y curl pkg-config meson ninja-build libgtk-4-dev libpango1.0-dev libcairo2-dev libglib2.0-dev libgdk-pixbuf-2.0-dev libgirepository1.0-dev valac libudev-dev libtss2-dev build-essential git
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN git clone https://github.com/wmww/gtk4-layer-shell.git /tmp/gtk4-layer-shell && cd /tmp/gtk4-layer-shell && meson setup build -Dvapi=false -Ddocs=false -Dintrospection=false && ninja -C build && ninja -C build install && ldconfig
EOF
podman build -t clippy_env -f Dockerfile.clippy .
podman run --rm -v "$PWD":/workspace -w /workspace clippy_env bash -c "cargo clippy --workspace --exclude ebpf-core --exclude ermete-sysmon-ebpf --all-targets --all-features -- -A clippy::undocumented_unsafe_blocks -A clippy::multiple_unsafe_ops_per_block -D warnings -A dead_code -A unused_variables"
