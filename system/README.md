<div align="center">
  <h1>🦅 Ermete OS — OCI Production Image & 4-Tier Pyramid Caching</h1>
  <p><b>The Golden Standard of Linux. An extreme, cloud-native, Zero-Maintenance Rolling Release desktop OS.</b></p>
</div>

---

The `system` sub-project is the final assembly line of Ermete OS. It takes the output artifacts from the `forge` (Custom RPMs) and the `kernel` (Chimera Kernel) sub-projects, and layers them sequentially on top of the **Fedora Atomic 43** base image to produce the final `ermete-os-system` bootable OCI container.

## 🏗️ Architecture: The 4-Tier Pyramid OCI Layering Strategy

Ermete OS structures the OS into **4 sequential layers** inside `Containerfile` to achieve maximum OCI layer caching efficiency. Lower tiers require reboots to update, while higher tiers can be updated live.

```
+-------------------------------------------------------------------------+
|                  TIER 3: AGILE RUST SHELL & APPS (~8 MB)                |
|           ermete-shell-rs, ermete-settings-rs, ermete-store-rs          |
|              [Layer 1 - Live Updateable without reboot]                 |
+-------------------------------------------------------------------------+
|                TIER 2: DESIGN SYSTEM & STATIC ASSETS (~18 MB)           |
|            Bibata Cursors, Matugen Dynamic Colors, Starship             |
+-------------------------------------------------------------------------+
|          TIER 1: DISPLAY SERVER & CORE USERSPACE SERVICES (~34 MB)      |
|         Cage Wayland Kiosk, Greetd, Niri Compositor, System Config      |
+-------------------------------------------------------------------------+
|          TIER 0: BEDROCK HARDWARE & KERNEL FOUNDATION (~3.3 GB)         |
|     Fedora Atomic 43 + Chimera Kernel + NVIDIA DKMS + SELinux Policies  |
|       [Bedrock Diet Applied: -1.1 GB Server Firmware & Build Pruned]    |
|                 [Layer 0 - Reboot Required for updates]                 |
+-------------------------------------------------------------------------+
```

### The Bedrock Diet (-1.1 GB Safe Pruning)
Inside Tier 0 and the final hardening step, Ermete OS applies the **Bedrock Diet** to strip non-consumer datacenter fat, drastically reducing the final OCI image size:
- **Server Firmware Removal (-400 MB)**: Purges `mellanox`, `qlogic`, `netronome`, `liquidio` datacenter network firmware blobs while keeping 100% of AMD/Intel/NVIDIA/Wi-Fi/BT consumer hardware firmware.
- **DKMS Build Tools Removal (-350 MB)**: Removes `kernel-devel`, `gcc`, and `make` after out-of-tree NVIDIA driver compilation.
- **DNF Cache Purge (-350 MB)**: Strips intermediate metadata from `/var/cache/dnf` and `/var/lib/dnf`.

---

## 🔐 The Big Tech Glassmorphism Login Greeter

Pre-login authentication is driven by **`ermete-shell-rs --greeter`** running inside a **Wayland Kiosk `cage`** session configured in `/etc/greetd/config.toml`:
- **Dynamic User Discovery**: Automatically inspects `/etc/passwd` to locate standard user accounts (`UID >= 1000`).
- **Glassmorphic UI**: Translucent cards, avatar frames loading `~/.face`, interactive **Caps Lock Indicator**, password reveal toggle, and integrated power menu.
- **Live Deployment**: Updated dynamically via `deploy-live-rust-greeter.sh` without rebooting.

---

## 🛠️ Build Pipeline & CI/CD Security (`system-build.yml`)

The final OS image is assembled automatically via GitHub Actions:

1. **Docker Buildx Assembly**: The `docker/build-push-action` constructs the image using the `Containerfile`. We intentionally bypass GitHub Action cache export (`cache-to: type=gha`) for this massive artifact to prevent GitHub Runner disk exhaustion (`No space left on device` or `BuildKit EOF` errors).
2. **Push to GHCR**: The image is compressed using advanced ZSTD (`compression=zstd,force-compression=true`) and pushed to the GitHub Container Registry.
3. **Trivy Vulnerability Scan**: The pushed image is scanned by **Aqua Security Trivy**. Since the `bootc` image contains no web application dependencies, the scanner focuses strictly on OS-level CVEs (both `os` and `library`).
4. **Cosign Cryptographic Signature**: The image is cryptographically signed using **Sigstore Cosign**, ensuring that end-users are booting an untampered, verified build of Ermete OS.

---

## 🚀 Bare Metal Deployment & Kickstart

### In-Place Atomic Switch (For existing Fedora Atomic/Silverblue systems)
```bash
sudo bootc switch ghcr.io/patapem/ermete-os-system:latest
```

### Automated ISO Build via `bootc-image-builder` (For clean installs)
```bash
sudo podman run --rm -it --privileged --pull=newer \
    --security-opt label=type:unconfined_t \
    -v $(pwd)/output:/output \
    -v $(pwd)/ermete-install.ks:/config.ks \
    quay.io/centos-bootc/bootc-image-builder:latest \
    --type iso --kickstart /config.ks \
    ghcr.io/patapem/ermete-os-system:latest
```
