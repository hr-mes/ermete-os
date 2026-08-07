<div align="center">
  <h1>🦅 Ermete OS — System Services, 4 God Nodes & OCI Layering Strategy</h1>
  <p><b>The Golden Standard of Linux. An extreme, cloud-native, Zero-Maintenance Rolling Release desktop OS.</b></p>
  <p>📚 <b><a href="../docs/architecture/doc_kernel_layer.md">Kernel Layer & Boot</a></b> | <b><a href="../docs/architecture/doc_core_daemons.md">Core Daemons & Security</a></b> | <b><a href="ARCHITECTURE.md">System Architecture & 4 God Nodes</a></b></p>
</div>

---

The `system` sub-project is the final assembly line and core runtime engine of Ermete OS. It takes the output artifacts from the `forge` (Custom RPMs) and the `kernel` (Chimera Kernel) sub-projects, and integrates the **4 God Nodes** on top of the **Fedora Atomic 43** base image to produce the final `ermete-os-system` bootable OCI container.

---

## 🏛️ The 4 System God Nodes

The `system/` directory hosts the 4 fundamental God Nodes driving system execution, hardware security, post-quantum networking, and app orchestration:

```
+-----------------------------------------------------------------------------------+
|               GOD NODE 4: FLATPAK DECLARATIVE ORCHESTRATOR                        |
|                        system/ermete-store                                        |
|         - SLSA Level 4 OCI App Management & Cosign Signature Verification        |
+-----------------------------------------------------------------------------------+
|               GOD NODE 3: MESH PQC (POST-QUANTUM CRYPTOGRAPHY BUS)                 |
|                        system/ermete-mesh-bus                                     |
|         - Dilithium5 / ML-DSA Signatures & ML-KEM / Kyber1024 WireGuard P2P       |
+-----------------------------------------------------------------------------------+
|               GOD NODE 2: MICRO-HYPERVISOR ENCLAVE ORCHESTRATOR                   |
|                        system/ermete-hypervisor-daemon                            |
|         - Hardware Confidential Computing: AMD SEV-SNP & Intel TDX Enclaves       |
+-----------------------------------------------------------------------------------+
|               GOD NODE 1: KERNEL AI SCHEDULER                                     |
|                        system/ermete-ebpf-sched                                   |
|         - Ring-0 sys_execve eBPF Tracepoints, NPU AI Prediction, sched_ext        |
+-----------------------------------------------------------------------------------+
```

### 1. 🧠 Kernel AI Scheduler ([`system/ermete-ebpf-sched`](file:///var/home/ermete/GEMINI/ermete-os/system/ermete-ebpf-sched))
- **Role:** AI-Driven eBPF Kernel Scheduler Bridge.
- **Implementation:** Intercepts `sys_execve` process creation events via eBPF, passes process metadata to `ermete-ai-daemon` on the NPU, and dynamically configures Ring-0 `sched_ext` task latency slice targets (100μs to 20ms) and cgroup v2 `cpu.weight`.

### 2. 🛡️ Micro-Hypervisor Enclave ([`system/ermete-hypervisor-daemon`](file:///var/home/ermete/GEMINI/ermete-os/system/ermete-hypervisor-daemon))
- **Role:** Zero-Trust Hardware Confidential Enclave Daemon.
- **Implementation:** Uses KVM, `vmm-sys-util`, AMD SEV-SNP, and Intel TDX extensions to launch and verify cryptographically sealed hardware enclaves for isolated execution of Ring-0 security workloads.

### 3. ⚡ Mesh PQC ([`system/ermete-mesh-bus`](file:///var/home/ermete/GEMINI/ermete-os/system/ermete-mesh-bus))
- **Role:** Post-Quantum Cryptography P2P WireGuard Mesh Bus.
- **Implementation:** Implements quantum-resistant key exchange (**ML-KEM-1024 / Kyber1024**) and digital signatures (**Dilithium5 / ML-DSA-87**) over peer-to-peer WireGuard tunnels and ZBus IPC, ensuring immunity against quantum decryption attacks.

### 4. 🏛️ Flatpak Declarative Orchestrator ([`system/ermete-store`](file:///var/home/ermete/GEMINI/ermete-os/system/ermete-store))
- **Role:** Declarative App Sandbox & OCI Container Engine.
- **Implementation:** Completely disconnects Flathub, using strict SLSA Level 4 OCI container images verified with **Cosign** signatures before installation.

---

## 🔥 The 5 Assimilated Proprietary Pillars (Pure Rust Re-writes)

In addition to the 4 God Nodes, `system/` hosts the **5 Native Assimilated Pillars** that devoured legacy C/C++ Linux subsystems:

1. **🪟 `ermete-compositor` ([`system/ermete-compositor`](file:///var/home/ermete/GEMINI/ermete-os/system/ermete-compositor))**: Pure Rust Wayland Compositor powered by Smithay (DRM/KMS, Udev, EGL) with an AI-driven dynamic window tiling engine. Replaces Mutter/Weston.
2. **🤖 `ermete-init-oracle` ([`system/ermete-init-oracle`](file:///var/home/ermete/GEMINI/ermete-os/system/ermete-init-oracle))**: Autonomous Tokio & Zbus AI systemd init oracle that monitors unit lifecycle and auto-heals failing services. Replaces C Systemd init.
3. **🎵 `ermete-audio-bus` ([`system/ermete-audio-bus`](file:///var/home/ermete/GEMINI/ermete-os/system/ermete-audio-bus))**: Pure Rust real-time PipeWire session manager and zero-copy audio swarm router. Replaces Pipewire/PulseAudio C daemons.
4. **🔑 `ermete-greeter` ([`system/ermete-greeter`](file:///var/home/ermete/GEMINI/ermete-os/system/ermete-greeter))**: Zero-Trust TPM 2.0 key release & hardware attestation display manager with `ZeroizeOnDrop` memory protection. Replaces greetd/gdm.
5. **🛡️ `xdg-desktop-portal-ermete` ([`forge/specs/ermete-xdg-desktop-portal-ermete`](file:///var/home/ermete/GEMINI/ermete-os/forge/specs/ermete-xdg-desktop-portal-ermete/xdg-desktop-portal-ermete-1.0.0))**: Native Rust Zbus 4.4 async desktop portal implementation for SLSA Level 4 Flatpak sandboxes. Replaces C XDG desktop portals.

---

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
|     Cage Wayland Kiosk, Greetd, Niri Compositor, ermete-mesh-bus        |
+-------------------------------------------------------------------------+
|          TIER 0: BEDROCK HARDWARE & KERNEL FOUNDATION (~3.3 GB)         |
|  Fedora Atomic 43 + Chimera Kernel + ermete-ebpf-sched + SEV-SNP / TDX  |
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
sudo bootc switch ghcr.io/hr-mes/ermete-os-system:latest
```

### Automated ISO Build via `bootc-image-builder` (For clean installs)
```bash
sudo podman run --rm -it --privileged --pull=newer \
    --security-opt label=type:unconfined_t \
    -v $(pwd)/output:/output \
    -v $(pwd)/ermete-install.ks:/config.ks \
    quay.io/centos-bootc/bootc-image-builder:latest \
    --type iso --kickstart /config.ks \
    ghcr.io/hr-mes/ermete-os-system:latest
```
