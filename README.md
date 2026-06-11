<div align="center">
  <h1>🦅 Ermete OS</h1>
  <p><b>An uncompromising, cloud-native, atomic Linux distribution engineered for power-users.</b></p>
</div>

---

**Ermete OS** is a hyper-optimized, immutable operating system built upon Fedora and Universal Blue (`bootc`) technologies. It discards monolithic desktop environments, replacing them with a surgically thin, keyboard-driven Wayland experience written almost entirely in Rust.

Driven by an absolute **Infrastructure-as-Code (IaC)** philosophy, Ermete OS is defined entirely by OCI container recipes. It guarantees unbreakable atomic updates, zero system entropy, and uncompromising privacy.

## 🏗️ Multi-Layer OCI Architecture
Ermete OS strictly follows a multi-repository, decoupled architecture for ultimate determinism.

```mermaid
graph TD
    subgraph Layer 0 [Base NVIDIA Repository]
        A[Fedora Base Atomic] --> B[Strip Vanilla Kernel]
        B --> C[Inject CachyOS Kernel BORE/x86-64-v3]
        C --> D[Compile NVIDIA DKMS via ld.bfd]
        D --> E[Dracut ZSTD Initramfs / Force KMS]
    end
    subgraph Layer 1 [Ermete OS Repository]
        E --> F[Containerfile FROM ghcr.io/.../ermete-base-nvidia]
        F --> G[Recipe: Rust Transient Build Pipeline]
        G --> H[Recipe: Firewalld Drop / Privacy Hardening]
        H --> I[Recipe: First-Boot Flatpak Provisioner]
    end
    Layer 1 --> J((Deployable Bootc Image))
```

## 🌟 The Enterprise Manifesto

1. **Zero-Entropy**: The root filesystem is strictly immutable. No system degradation, rot, or state drift over time.
2. **Zero-Bloat**: Only CLI tools and core infrastructure exist on the host. Every GUI application is strictly sandboxed via Flatpak. Weak dependencies are mathematically banned (`install_weak_deps=False`).
3. **100% Verified Supply Chain**: External binaries are either cryptographically verified against pinned SHA256 checksums or dynamically compiled from source (via Cargo) within a transient build layer that purges all compilation tools (gcc, rust, devel packages) before sealing the OCI image.
4. **Cryptographic Signatures**: Every deployment image is signed via Keyless OIDC (Sigstore/Cosign). Client machines automatically verify cryptographic authenticity, preventing downgrade attacks.

---

## 🛡️ Paranoid Privacy & The "Zero Denial of Service" Doctrine
Ermete OS implements extreme enterprise-grade security defaults, but aggressively balances them against usability to ensure user workflows are never paralyzed by their own defenses.

### 🎯 Threat Model & Usability Balance
- **Network Surveillance vs UX**: DNS-over-TLS (DoT) is enforced, but set to `opportunistic` to prevent bricking the system on restrictive networks. MAC Address Randomization is set to `stable`—preventing physical tracking while guaranteeing survival on Captive Portals (hotels/airports) and standard DHCP environments.
- **Remote Exploitation vs Discovery**: Neutralized via a Zero-Trust `drop` zone Firewalld policy. Port scanning yields zero responses, but `mdns` is surgically whitelisted to guarantee local network device discovery (Chromecast, Wireless Printers).
- **Compromised GUI Software vs Compatibility**: Neutralized via Flatpak sandboxing and fine-grained `Flatseal` permission management, avoiding global overrides that would silently break XWayland compatibility or filesystem access for non-native applications.
- **Post-Exploitation Data Leaks**: Neutralized by disabling systemd coredumps (`Storage=none`), preventing RAM secrets from being flushed to unencrypted disk sectors upon application crashes.
- **Local Privilege Escalation**: Mitigated by aggressive kernel hardening (`kptr_restrict=2`, `dmesg_restrict=1`, unprivileged BPF disabled). User skeletons (`/etc/skel`) apply fine-grained privacy permissions (`chmod 600/700`) without stripping execution flags from vital scripts.

---

## ⚡ Extreme Performance & Architecture
- **ZRAM Compressed Memory**: Out-of-the-box `zram-generator` configuration uses **ZSTD compression** and allocates up to 100% of RAM dynamically (`vm.swappiness=150`), maximizing multitasking fluidity.
- **OOMD Wayland Protection**: `systemd-oomd` relies on balanced system defaults. Aggressive memory limits are explicitly avoided to prevent "Nuke Traps" where the entire Wayland graphical cgroup collapses under a browser RAM spike.
- **TCP BBR & Network**: The kernel is tuned to use `fq_pie` and Google's `bbr` congestion control algorithm, minimizing bufferbloat and latency.
- **Silent & Lightning Boot**: Legacy Dracut modules (floppy, pcsc) are omitted. Kernel arguments are tuned (`quiet`, `loglevel=3`) for a completely silent, high-speed boot.
- **Greenboot Auto-Repair**: Non-blocking `greenboot` health checks monitor Wayland (`greetd`) and Network services. If a critical failure occurs, the OS automatically and seamlessly rolls back to the previous working OSTree deployment.

---

## 🎨 The "Full-Rust" Wayland Stack
The graphical layer abandons monolithic desktops in favor of individual, hyper-fast components built in memory-safe Rust. All components are themed using the **Catppuccin Mocha** palette, **Inter** UI fonts, and **JetBrains Mono**.

- **Compositor**: [Niri](https://github.com/YaLTeR/niri) (Scrollable Tiling Wayland Compositor).
- **Status Bar**: [Ironbar](https://github.com/JakeStanger/ironbar) (Cryptographically verified).
- **App Launcher**: [Anyrun](https://github.com/anyrun-org/anyrun) (Compiled offline via transient layer).
- **Login Greeter**: `tuigreet` (Terminal-based greeter with password masking).
- **Terminal**: `Alacritty` (GPU-accelerated, zero-latency emulator).
- **Core Utilities**: Rust replacements for UNIX tools (`eza`, `bat`, `fd-find`, `ripgrep`, `bottom`, `nushell`, `starship`).
- **Terminal IDE**: **Neovim** is pre-configured with **LazyVim** (Catppuccin Mocha theme).

---

## 📦 Automated Flatpak Ecosystem (First-Boot Provisioner)
To preserve host purity, **no graphical software is installed via system packages**.
Upon your very first boot and network connection, Ermete OS provisions your user-space via an intelligent, non-blocking `oneshot` systemd service (`Ermete-firstboot.service`). It globally enforces the **Adwaita Dark** GTK theme and **Papirus** icons across all sandboxes, and automatically installs:

1. **Alacritty**, **Flatseal**, **Warehouse**, **Firefox**, **Nautilus**, **MPV**, **OBS Studio**.

*All Flatpaks leverage the proprietary NVIDIA drivers and RPMFusion codecs seamlessly injected into the base image. The provisioner features infinite-loop idempotency, ensuring offline bootups or captive portals never permanently break the installation sequence.*

---

## 🛠️ Modifying the OS (IaC)
Ermete OS abandons post-install scripts. To modify the OS, simply edit the `Containerfile` or the modular bash recipes in `build_files/recipes/`. The recipes are isolated in discrete `RUN` layers to maximize OCI cache hits during development.

### Local Build & Testing
Make sure you have `podman` and `just` installed:
- `just build`: Builds the container image locally.
- `just build-iso`: Generates a bootable ISO.
- `just run-vm-qcow2`: Compiles a QCOW2 image and boots it instantly via QEMU for rapid testing.
