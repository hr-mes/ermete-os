<div align="center">
  <h1>🦅 Ermete OS (Layer 1: Ring 3)</h1>
  <p><b>The Golden Standard of Linux. An extreme, cloud-native, Zero-Maintenance Rolling Release desktop OS.</b></p>
</div>

---

**Ermete OS** is a hyper-modern, immutable Operating System distributed as a Bootable OCI Container.
It merges the absolute performance of CachyOS with the bulletproof atomicity of Fedora OSTree, wrapping everything in a stunning, macOS-grade UI orchestrated by AGS and Niri.

## 🏗️ The Bedrock Forge Ecosystem (Absolute RPM Encapsulation)

In strict adherence to the **"Absolute RPM Encapsulation"** dogma, Ermete OS **contains zero raw scripts or scattered files** (the `system_files/` pattern has been eradicated). The entire OS is conceived as a pure declarative OCI layer that mathematically merges pre-forged components.

The "Raven's View" architecture follows a flawless OCI waterfall:
1. **Layer 0 (Base NVIDIA)**: Formal derivation from `ghcr.io/patapem/ermete-base-nvidia:latest`. It provides the Fedora 43 base, the PGO-optimized Trismegistus Kernel, and NVIDIA DKMS drivers, chronologically synced for perfect KMS early-boot initialization.
2. **Layer 1 (The UI Graveyard)**: The OS extracts the topological graph from GHCR. All components (Astal, Hyprpanel, AGS v1/v2, Starship, Matugen, Niri config, system scripts) originate strictly from micro-containers (`ghcr.io/patapem/ermete-forge-*`) compiled in the private Forge.
3. **Layer 2 (Upstream Injection)**: Modern tools (`eza`, `bat`, `nushell`, `ripgrep`) are fetched from the Forge's rolling-upstream pipeline, compiled aggressively with GCC 15 `x86-64-v3` and `-O3`.
4. **The Final Bake (Zero-Bloat)**: DNF5 resolves the dependency graph (`dnf5 install --allowerasing`), overriding older Fedora packages with the extreme Forge RPMs, producing a slim, immaculate filesystem with zero ghost layers.

All custom packages are autonomously built, linted, and cryptographically signed in the adjacent [Ermete Forge](https://github.com/patapem/ermete-forge) repository.

## ⚡ Extreme Performance & Wayland Stack
- **ZRAM Compressed Memory**: 100% RAM allocation dynamically compressed via **ZSTD** (`vm.swappiness=150`).
- **Systemd User Orchestration**: The Wayland compositor (Niri) does not spawn processes imperatively. Everything is handled cleanly by `systemd --user` binding to `niri-session.target`, ensuring graceful teardown and infinite idempotency.
- **The Stack**:
  - Compositor: **Niri** (Scrollable Tiling). Hardware accelerated with `GBM_BACKEND=nvidia-drm`.
  - UI Framework: **AGS (Aylur's Gtk Shell)**. A unified TypeScript/GJS ecosystem providing a 360Hz "Super Premium UX" for Top Bar, Control Center, App Launcher, Notifications, and Power Menu.
  - Terminal: **Foot** (Wayland native, C-based, lightweight).

## ⚖️ Le Leggi del Ring 3 (Project Rules)
Queste direttive generali assicurano la coerenza architetturale, garantendo che lo strato utente rimanga robusto e isolato dal core del sistema.

### 🟢 COSA DEVE ESSERE FATTO (The "Musts")
- **PIPELINE:** Ereditare in modo immutabile l'infrastruttura sottostante di Base NVIDIA, agendo esclusivamente come aggregatore di livello superiore.
- **PIPELINE:** Gestire le sovrascritture di configurazione in modo formale e dichiarativo, permettendo ai pacchetti custom di sostituire le logiche legacy in fase di assemblaggio.
- **USERSPACE:** Mantenere il root filesystem pristino e intoccato a runtime. App e framework devono girare in sandboxing (Flatpak) o in ambienti dinamicamente isolati (Nix).

### 🔴 COSA NON DEVE ESSERE MAI FATTO (The "Never Do's")
- **PIPELINE:** Utilizzare comandi shell distruttivi all'interno del processo di generazione dell'immagine (es. sed, cp, rm) per manipolare configurazioni di sistema; queste devono derivare da pacchetti strutturati.
- **PIPELINE:** Alterare o ricostruire componenti del Ring 0 in questo livello (la gestione hardware e boot appartiene esplicitamente al layer Base).
- **USERSPACE:** Effettuare modifiche dirette ai file globali post-installazione, aggirando il flusso di forgiatura. Ogni evoluzione del sistema deve nascere come modifica del codice sorgente.

## 📦 Segregated Software Management
Due to root immutability, the traditional `dnf install` paradigm is obliterated in user-space:
1. **Graphical Applications**: Exclusively confined to **Flatpak** (via Flathub). Protected globally by `flatpak override --system --device=dri --socket=wayland` to guarantee NVIDIA GPU acceleration and Wayland IPC sandboxing.
2. **CLI Utilities & Compilers**: Managed dynamically and declaratively via the **Nix** Package Manager (seamlessly exposed in `/etc/profile.d/nix.sh`), with an ephemeral state rigorously managed by systemd tmpfiles.

## 🔄 Zero-Maintenance Rolling Release
Ermete OS is an autonomous living organism:
1. **Upstream Monitoring**: Ermete Forge continuously monitors Fedora and GitHub for updates.
2. **Horizontal Triggering**: When the Forge recompiles a package (e.g. Kernel or AGS), it fires a `repository_dispatch` API call.
3. **Automated Rebuild**: Base-NVIDIA and Ermete OS rebuild themselves automatically based on the dependency chain, using immutable digests (`skopeo` inspection).
4. **OTA Updates**: Your PC downloads the cryptographically verified (Cosign) image in the background and applies it atomically on the next reboot. You do absolutely nothing.

## 🚀 Deployment (Bare Metal Installation)

### In-Place Mutation
If you are currently running Fedora Workstation or Silverblue, atomically mutate your root filesystem:
```bash
sudo bootc switch ghcr.io/patapem/ermete-os
```

### Zero-Touch Provisioning (Kickstart)
The repository includes a ready-to-use `ermete-install.ks` Kickstart file, designed for advanced power-users.
To generate the ISO using `bootc-image-builder`:
```bash
sudo podman run \
    --rm -it --privileged --pull=newer \
    --security-opt label=type:unconfined_t \
    -v $(pwd)/output:/output \
    -v $(pwd)/ermete-install.ks:/config.ks \
    quay.io/centos-bootc/bootc-image-builder:latest \
    --type iso --kickstart /config.ks \
    ghcr.io/patapem/ermete-os:latest
```
