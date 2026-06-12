<div align="center">
  <h1>🦅 Ermete OS (Layer 1: Ring 3)</h1>
  <p><b>An uncompromising, cloud-native, atomic Linux distribution engineered for power-users.</b></p>
</div>

---

**Ermete OS** is a hyper-optimized, immutable operating system built upon Fedora and Universal Blue (`bootc`) technologies. It discards monolithic desktop environments, replacing them with a surgically thin, keyboard-driven Wayland experience written almost entirely in Rust.

Driven by an absolute **Infrastructure-as-Code (IaC)** philosophy, Ermete OS is defined entirely by OCI container recipes. It guarantees unbreakable atomic updates, zero system entropy, and uncompromising privacy.

## 🏗️ Multi-Layer OCI Architecture
Ermete OS strictly follows a multi-repository, decoupled architecture for ultimate determinism.

```mermaid
graph TD
    subgraph Layer 0 [Base NVIDIA Repository / Ring 0]
        A[Fedora Base Atomic] --> B[Inject CachyOS Kernel]
        B --> C[Compile NVIDIA DKMS via ld.bfd]
        C --> D[Nightly CI/CD & Cosign Signature]
    end
    subgraph Layer 1 [Ermete OS Repository / Ring 3]
        D -- "API Dispatch Trigger" --> E[Containerfile FROM ghcr.io/.../ermete-base-nvidia]
        E --> F[Inject Renovate ARGs (IaC Single Source of Truth)]
        F --> G[Recipe: Rust Transient Build Pipeline]
        G --> H[Recipe: Firewalld Drop / Privacy Hardening]
        H --> I[Recipe: Extracted Systemd Flatpak Provisioner]
    end
    Layer 1 --> J((Deployable Bootc Image))
```

## 🌟 The Enterprise Manifesto

1. **Zero-Entropy**: The root filesystem is strictly immutable. No system degradation, rot, or state drift over time. Software installation via `dnf` on the live system is mathematically banned.
2. **Zero-Bloat**: Only CLI tools and core infrastructure exist on the host. Weak dependencies are banned (`install_weak_deps=False`).
3. **100% Verified Supply Chain**: External binaries (Ironbar, Starship, Anyrun) are managed via a centralized `Containerfile ARG` manifest. They are either cryptographically verified against pinned SHA256 checksums or dynamically compiled from source (via Cargo) within a transient build layer that purges all compilation tools before sealing the OCI image.
4. **Autonomous Maintenance (Renovate Bot)**: The OS "heals and updates itself." Renovate Bot parses the `Containerfile` ARGs, detects new upstream releases, recalculates the SHA256 hashes, and opens Pull Requests. The Architect (User) merely approves the PR, triggering the CI/CD pipeline to build and cryptographically sign the new immutable deployment.

---

## 🛡️ Paranoid Privacy & The "Zero Denial of Service" Doctrine
Ermete OS implements extreme enterprise-grade security defaults, aggressively balanced against usability to ensure workflows are never paralyzed by their own defenses.

- **Network Surveillance vs UX**: DNS-over-TLS (DoT) is enforced but set to `opportunistic`. MAC Address Randomization is `stable`—preventing physical tracking while surviving Captive Portals.
- **Remote Exploitation vs Discovery**: Neutralized via a Zero-Trust `drop` zone Firewalld policy. `mdns` is surgically whitelisted to guarantee local device discovery (Chromecast, Wireless Printers).
- **Post-Exploitation Data Leaks**: Systemd coredumps are disabled (`Storage=none`), preventing RAM secrets from bleeding onto the disk upon application crashes.
- **Local Privilege Escalation**: Mitigated by aggressive kernel hardening (`kptr_restrict=2`, unprivileged BPF disabled) and granular `/etc/skel` permissions (`chmod 600/700`).

---

## 🎨 The "Premium Ricing" Aesthetic (Catppuccin Mocha)
The graphical layer abandons standard Linux grayness. The UI is designed to evoke a "Wow" factor immediately, blending cyberpunk minimalism with modern Apple-like glassmorphism.
- **Color Palette**: Strict adherence to the highly acclaimed **Catppuccin Mocha** dark mode palette (`#1e1e2e` backgrounds, `#89b4fa` accents).
- **Dynamic UI**: Components float via deep `box-shadow` configurations and `border-radius: 12px/16px`. Hover and focus events feature `0.2s ease-in-out` micro-animations for an organic, tactile feel.
- **Typography**: **Inter** for absolute UI legibility, **Font Awesome 6** for vector iconography, and **JetBrains Mono** for developer terminal superiority.

## ⚡ Extreme Performance & Wayland Stack
- **ZRAM Compressed Memory**: 100% RAM allocation dynamically compressed via **ZSTD** (`vm.swappiness=150`).
- **OOMD Wayland Protection**: Aggressive memory limits are avoided to prevent "Nuke Traps" where the Wayland cgroup collapses under browser RAM spikes.
- **The Stack**:
  - Compositor: **Niri** (Scrollable Tiling).
  - Status Bar: **Ironbar** (Floating, transparent).
  - App Launcher: **Anyrun** (Compiled offline).
  - Terminal: **Alacritty** (GPU-accelerated).
  - IDE: **Neovim** (LazyVim Catppuccin).

---

## 📦 Segregated Software Management
Due to root immutability, the traditional `.exe` or `dnf install` paradigm is obliterated. Users must adapt to compartmentalized software deployments:
1. **Graphical Applications**: Exclusively confined to **Flatpak** (via Flathub).
2. **CLI Utilities**: Managed via **Nix** or **Homebrew** (Zero-Trust mapped into `/var`).
3. **Destructive Experiments**: Handled via integrated **Distrobox** containers (e.g., disposable Arch Linux shells).

To preserve host purity, upon the very first boot and network connection, a non-blocking `oneshot` systemd service (`Ermete-firstboot.service`) silently executes a discrete script (`/usr/libexec/ermete-firstboot.sh`). It globally enforces the **Adwaita Dark** GTK theme, **Papirus-Dark** icons, and automatically installs essential Flatpaks (Warehouse, Flatseal, Firefox, MPV) with infinite-loop idempotency.

---

## 🛠️ Modifying the OS (IaC)
You do not modify this OS from the terminal. You sculpt it from the Cloud.
To change versions or add packages, edit the `Containerfile` or the bash recipes in `build_files/recipes/` and push to GitHub. 

### Local Testing
Make sure you have `podman` and `just` installed:
- `just build`: Builds the container image locally.
- `just run-vm-qcow2`: Compiles a QCOW2 image and boots it instantly via QEMU for rapid testing.
