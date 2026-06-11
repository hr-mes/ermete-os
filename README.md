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
        D --> E[Dracut ZSTD Initramfs]
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

## 🛡️ Paranoid Privacy & Security Fortress
Ermete OS implements extreme enterprise-grade defaults to protect user data and telemetry.

### 🎯 Threat Model Definition
- **Network Surveillance & ISP Tracking**: Neutralized via system-wide DNS-over-TLS (DoT) and physical MAC Address Randomization.
- **Remote Exploitation**: Neutralized via a Zero-Trust `drop` zone Firewalld policy applied by default. Port scanning yields zero responses.
- **Malicious/Compromised GUI Software**: Neutralized via draconian Flatpak sandboxing (X11 sockets permanently disabled, `~/.home` directory access denied, forcing XDG portals).
- **Post-Exploitation Data Leaks**: Neutralized by disabling systemd coredumps (`Storage=none`), preventing RAM secrets from being flushed to unencrypted disk sectors upon application crashes.
- **Local Privilege Escalation**: Mitigated by aggressive kernel hardening (`kptr_restrict=2`, `dmesg_restrict=1`, unprivileged BPF disabled) and strict SELinux MAC enforcement. `/etc/skel` applies draconian `go-rwx` permissions.

---

## ⚡ Extreme Performance & Architecture
- **ZRAM Compressed Memory**: Out-of-the-box `zram-generator` configuration uses **ZSTD compression** and allocates up to 100% of RAM dynamically (`vm.swappiness=150`), maximizing multitasking fluidity.
- **OOMD Wayland Protection**: `systemd-oomd` is aggressively tuned (90% limit / 5 seconds threshold) to surgically kill memory-hogging processes *before* the Wayland session freezes.
- **TCP BBR & Network**: The kernel is tuned to use `fq_pie` and Google's `bbr` congestion control algorithm, minimizing bufferbloat and latency.
- **Silent & Lightning Boot**: Legacy Dracut modules (floppy, pcsc) are omitted. Kernel arguments are tuned (`quiet`, `loglevel=3`) for a completely silent, high-speed boot.
- **Greenboot Auto-Repair**: Non-blocking `greenboot` health checks monitor Wayland (`greetd`) and Network services. If a critical failure occurs, the OS automatically and seamlessly rolls back to the previous working OSTree deployment.

---

## 🎨 The "Full-Rust" Wayland Stack
The graphical layer abandons monolithic desktops in favor of individual, hyper-fast components built in memory-safe Rust. All components are themed using the **Catppuccin Mocha** palette, **Inter** UI fonts, and **JetBrains Mono**.

- **Compositor**: [Niri](https://github.com/YaLTeR/niri) (Scrollable Tiling Wayland Compositor).
- **Status Bar**: [Ironbar](https://github.com/JakeStanger/ironbar) (Natively compiled).
- **App Launcher**: [Anyrun](https://github.com/anyrun-org/anyrun) (Natively compiled).
- **Login Greeter**: `tuigreet` (Terminal-based greeter with password masking).
- **Terminal**: `Alacritty` (GPU-accelerated, zero-latency emulator).
- **Core Utilities**: Rust replacements for UNIX tools (`eza`, `bat`, `fd-find`, `ripgrep`, `bottom`, `nushell`, `starship`).
- **Terminal IDE**: **Neovim** is pre-configured with **LazyVim** (Catppuccin Mocha theme).

---

## 📦 Automated Flatpak Ecosystem (First-Boot Provisioner)
To preserve host purity, **no graphical software is installed via system packages**.
Upon your very first boot and network connection, Ermete OS provisions your user-space via an intelligent, non-blocking `oneshot` systemd service (`Ermete-firstboot.service`). It globally enforces the **Adwaita Dark** GTK theme and **Papirus** icons across all sandboxes, and automatically installs:

1. **Alacritty**, **Flatseal**, **Warehouse**, **Firefox**, **Nautilus**, **MPV**, **OBS Studio**.

*All Flatpaks leverage the proprietary NVIDIA drivers and RPMFusion codecs seamlessly injected into the base image. The provisioner operates robustly, ensuring failures on captive portals do not permanently block installation.*

---

## 🛠️ Modifying the OS (IaC)
Ermete OS abandons post-install scripts. To modify the OS, simply edit the `Containerfile` or the modular bash recipes in `build_files/recipes/`. The recipes are isolated in discrete `RUN` layers to maximize OCI cache hits during development.

### Local Build & Testing
Make sure you have `podman` and `just` installed:
- `just build`: Builds the container image locally.
- `just build-iso`: Generates a bootable ISO.
- `just run-vm-qcow2`: Compiles a QCOW2 image and boots it instantly via QEMU for rapid testing.
