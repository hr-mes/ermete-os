#!/bin/bash
set -e
echo "Installing rustup..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
echo "Installing system dependencies..."
sudo apt-get update
sudo apt-get install -y pkg-config meson ninja-build libgtk-4-dev libpango1.0-dev libcairo2-dev libglib2.0-dev libgdk-pixbuf-2.0-dev libgirepository1.0-dev valac libudev-dev libtss2-dev build-essential
echo "Installing gtk4-layer-shell..."
if [ ! -d "/tmp/gtk4-layer-shell" ]; then
    git clone https://github.com/wmww/gtk4-layer-shell.git /tmp/gtk4-layer-shell
    cd /tmp/gtk4-layer-shell
    meson setup build -Dvapi=false -Ddocs=false -Dintrospection=false
    ninja -C build
    sudo ninja -C build install
    sudo ldconfig
fi
cd "/mnt/c/Users/Hermes/Documents/Gemini Project/ermete-os"
echo "Running cargo clippy..."
cargo clippy --workspace --exclude ebpf-core --exclude ermete-sysmon-ebpf --all-targets --all-features -- -A clippy::undocumented_unsafe_blocks -A clippy::multiple_unsafe_ops_per_block -D warnings -A dead_code -A unused_variables
