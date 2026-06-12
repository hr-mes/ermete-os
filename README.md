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
        A["Fedora Base Atomic"] --> B["Inject CachyOS Kernel"]
        B --> C["Compile NVIDIA DKMS via ld.bfd"]
        C --> D["Nightly CI/CD & Cosign Signature"]
    end
    subgraph Layer 1 [Ermete OS Repository / Ring 3]
        D -- "API Dispatch Trigger" --> E["Containerfile FROM ghcr.io/.../ermete-base-nvidia@sha256"]
        E --> F["Inject Renovate ARGs (IaC Single Source of Truth)"]
        F --> G["Recipe: Rust Transient Build Pipeline"]
        G --> H["Recipe: Firewalld Drop / Privacy Hardening"]
        H --> I["Recipe: Asynchronous Systemd Provisioner"]
    end
    Layer 1 --> J(("Deployable Bootc Image"))
```

## 🌟 The Enterprise Manifesto

1. **Zero-Entropy**: The root filesystem is strictly immutable. No system degradation, rot, or state drift over time. Software installation via `dnf` on the live system is mathematically banned.
2. **Zero-Bloat**: Only CLI tools and core infrastructure exist on the host. Weak dependencies are banned (`install_weak_deps=False`).
3. **100% Verified Supply Chain**: Dynamic `curl | bash` or blind binary downloads are forbidden. External binaries (Ironbar, Starship, Anyrun) are managed via a centralized `Containerfile ARG` manifest and verified against pinned SHA256 checksums. Furthermore, `tar` extractions are surgically hardened with `--no-same-owner` to prevent Local Privilege Escalation (LPE) via malicious UID injections.
4. **Autonomous Maintenance (Zero-Touch Rolling Release)**: The OS heals and updates itself. Renovate Bot parses the `Containerfile`, detects new upstream releases, recalculates SHA256 hashes, and pins Docker images to immutable digests. The pipeline triggers automatically, compiling and signing the new immutable deployment via Sigstore/Cosign OIDC without manual intervention.

---

## 🛡️ Paranoid Privacy & The "Zero Denial of Service" Doctrine
Ermete OS implements extreme enterprise-grade security defaults, aggressively balanced against usability to ensure workflows are never paralyzed by their own defenses.

- **Network Surveillance vs UX**: DNS-over-TLS (DoT) is enforced but set to `opportunistic`. MAC Address Randomization is `stable`—preventing physical tracking while surviving Captive Portals.
- **Remote Exploitation vs Discovery**: Neutralized via a Zero-Trust `drop` zone Firewalld policy. `mdns` is surgically whitelisted to guarantee local device discovery (Chromecast, Wireless Printers).
- **Post-Exploitation Data Leaks**: Systemd coredumps are disabled (`Storage=none`), preventing RAM secrets from bleeding onto the disk upon application crashes.
- **Zero-Trust UNIX Sandboxing**: Mitigated by aggressive kernel hardening (`kptr_restrict=2`, unprivileged BPF disabled). At the user level, `/etc/skel` permissions are ruthlessly locked down (`chmod 700` for directories), ensuring privacy-by-design for every newly provisioned account.

---

## ⚡ Extreme Performance & Wayland Stack
- **ZRAM Compressed Memory**: 100% RAM allocation dynamically compressed via **ZSTD** (`vm.swappiness=150`).
- **Network Latency Minimization**: Kernel TCP congestion control is forced to **BBR** combined with `fq_pie` queuing discipline for maximum throughput and minimum bufferbloat.
- **OOMD Wayland Protection**: Aggressive memory limits are avoided to prevent "Nuke Traps" where the Wayland cgroup collapses under browser RAM spikes.
- **The Stack**:
  - Compositor: **Niri** (Scrollable Tiling). Bootstrapped flawlessly via `niri-session` for perfect DBus and XDG-Desktop-Portal integration.
  - Status Bar: **Ironbar** (Floating, transparent).
  - App Launcher: **Anyrun** (Compiled offline dynamically).
  - Terminal: **Alacritty** (GPU-accelerated).
  - Environment: Complete Wayland/NVIDIA variable injection (`MOZ_ENABLE_WAYLAND=1`, `LIBVA_DRIVER_NAME=nvidia`).

---

## 📦 Segregated Software Management
Due to root immutability, the traditional `.exe` or `dnf install` paradigm is obliterated. Users must adapt to compartmentalized software deployments:
1. **Graphical Applications**: Exclusively confined to **Flatpak** (via Flathub).
2. **CLI Utilities**: Managed via **Nix** or **Homebrew** (Zero-Trust mapped into `/var`).
3. **Destructive Experiments**: Handled via integrated **Distrobox** containers (e.g., disposable Arch Linux shells).

To preserve host purity, the system employs an asynchronous, non-blocking `oneshot` systemd service (`Ermete-firstboot.service`) to install flatpaks. By decoupling the OS boot from the `NetworkManager-wait-online.service`, Ermete OS boots to the desktop in seconds, while Flatseal and core apps are provisioned silently in the background via infinite-loop idempotency.

---

## 🛠️ Modifying the OS (IaC)
You do not modify this OS from the terminal. You sculpt it from the Cloud.
To change versions or add packages, edit the `Containerfile` or the bash recipes in `build_files/recipes/` and push to GitHub. 

### Local Testing
Make sure you have `podman` and `just` installed:
- `just build`: Builds the container image locally.
- `just run-vm-qcow2`: Compiles a QCOW2 image and boots it instantly via QEMU for rapid testing.

---

## 🚀 Deployment (Bare Metal Installation)
Ermete OS is a bootable OCI container. To install it on physical hardware, use one of the following methods:

### Option 1: In-Place Mutation (Recommended)
If you are currently running a standard Fedora distribution (like Fedora Workstation or Silverblue), you can atomically mutate your root filesystem into Ermete OS:
```bash
sudo bootc switch ghcr.io/patapem/ermete
```
*Note: Depending on your exact ghcr.io path, verify the repository name.*

### Option 2: Generate a Bootable ISO
You can generate a standard `.iso` installer locally using `bootc-image-builder`:
```bash
sudo podman run \
    --rm \
    -it \
    --privileged \
    --pull=newer \
    --security-opt label=type:unconfined_t \
    -v $(pwd)/output:/output \
    quay.io/centos-bootc/bootc-image-builder:latest \
    --type iso \
    ghcr.io/patapem/ermete:latest
```
Burn the resulting ISO in the `output/` folder to a USB drive and install normally.
