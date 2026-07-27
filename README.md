<div align="center">
  <h1>🦅 Ermete OS Monorepo</h1>
  <p><b>The Golden Standard of Linux. An extreme, cloud-native, Zero-Maintenance Rolling Release desktop OS.</b></p>
</div>

---

Ermete OS is an immutable, hyper-optimized Operating System built on top of Fedora Atomic 43. It utilizes OCI container standards (`bootc`) to deliver a robust, unbreakable, and ultra-fast desktop experience. This monorepo contains the entire source code, build pipelines, and documentation required to build the OS from scratch.

## 🏗️ Repository Architecture

The monorepo is divided into three distinct, highly decoupled sub-projects. Each sub-project has its own dedicated GitHub Actions CI/CD pipeline, ensuring parallel compilation and isolated artifact generation.

### 1. 🌋 [Ermete Forge](./forge/README.md)
The **Private OCI Micro-Container & RPM Forge**. 
This is the package builder for Ermete OS. It enforces aggressive compiler optimizations (`-O3`, `-march=x86-64-v3`, `-flto=auto`, `mold` linker) across all custom packages (such as the 100% Rust UI Stack, Niri wrappers, and SELinux policies). 
Instead of a monolithic build, every single package is built in an isolated `scratch` container and exported as an RPM artifact. These packages are organized in a 4-Tier dependency system.

### 2. ⚡ [Ermete Kernel](./kernel/README.md)
The **Chimera Kernel Build System**.
This subsystem takes the high-performance patches from CachyOS (BORE/EEVDF schedulers, BBR3, ThinLTO) and compiles a bespoke, highly optimized kernel (the "Chimera Kernel") specifically tuned for Ermete OS. The kernel is packaged as a standard RPM, ready to be ingested by the final system image.

### 3. 💿 [Ermete System](./system/README.md)
The **Final OCI Production Image**.
This is the assembly line. It takes the base Fedora Atomic 43 image, injects the Chimera Kernel, applies the "Bedrock Diet" (stripping 1.1 GB of server firmware and compilation fat), and layers the custom RPMs from the Forge. The final result is the `ermete-os-system` OCI image, which is scanned by Trivy for vulnerabilities, signed by Cosign, and published to the GitHub Container Registry (GHCR).

---

## 🚀 Key Features

* **Absolute RPM Encapsulation**: Zero scattered configuration scripts. Every tweak, udev rule, and UI component is encapsulated in a clean `.spec` and `.rpm`.
* **100% Rust UI Stack**: JavaScript/GJS/NodeJS engines are strictly prohibited in user-space UI. The OS uses a custom Rust-based Glassmorphic Kiosk Greeter and Desktop Environment.
* **Immutable & Atomic**: Delivered as a `bootc` OCI image. Updates are transactional; if a system update fails, you can seamlessly rollback to the previous deployment.
* **High-Performance**: Out-of-the-box CachyOS kernel optimizations, ZRAM/Virtual memory sysctl tweaks, and process priority daemon (`ananicy`).

## 🛠️ Build Pipelines (GitHub Actions)

Ermete OS uses three primary workflows:
1. **Forge Build**: Compiles RPMs via Buildah/Podman and exports them.
2. **Kernel Build**: Compiles the Chimera Kernel with `ccache` for extreme speed.
3. **System Image Build**: Assembles the final OCI container, bypassing Buildx caching bottlenecks to prevent runner disk exhaustion, and securely pushes the signed image to GHCR.
