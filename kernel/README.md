# Ermete OS Kernel

This directory contains the build instructions and configuration for the **Chimera Kernel**, which is Ermete OS's custom kernel based on CachyOS sources but compiled via Fedora ARK and RPM specifications.

## Architecture
- **Spec File:** `ermete-kernel.spec` defines the RPM build process.
- **Config:** `ermete-bedrock.cfg` contains the specific kconfig overrides for our hardware and security footprint.
- **Build Script:** `build-kernel-rpm.sh` is the unified script that fetches sources, applies our bedrock configuration, and invokes `rpmbuild`.

*Note: All legacy Arch Linux / PKGBUILD scripts have been removed to maintain architectural purity.*
