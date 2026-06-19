#!/usr/bin/env bash
set -eoux pipefail

podman build --pull=newer --tag localhost/ermete-os:latest .
BUILDTMP=$(mktemp -p "$HOME" -d -t _build-bib.XXXXXXXXXX)

sudo podman run \
  --rm \
  -t \
  --privileged \
  --pull=newer \
  --net=host \
  --security-opt label=type:unconfined_t \
  -v "$(pwd)/disk_config/disk.toml:/config.toml:ro" \
  -v $BUILDTMP:/output \
  -v /var/lib/containers/storage:/var/lib/containers/storage \
  quay.io/centos-bootc/bootc-image-builder:latest \
  --type qcow2 --use-librepo=True --rootfs=btrfs --config /config.toml \
  localhost/ermete-os:latest

echo "Converting qcow2 to vhdx safely in native ext4 filesystem..."
sudo qemu-img convert -f qcow2 -O vhdx $BUILDTMP/qcow2/disk.qcow2 $BUILDTMP/disk.vhdx

mkdir -p output
sudo mv -f $BUILDTMP/* output/
sudo rmdir $BUILDTMP
sudo chown -R $USER:$USER output/
