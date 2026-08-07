# 🌋 Ermete Forge — Private OCI Micro-Container & RPM Forge

> 📚 **Deep-Dive Technical Documentation:** Per l'architettura dettagliata della pipeline CI/CD, l'idempotenza e il meccanismo deterministico Nix-Hermetic, consulta **[Build System & CI/CD Pipeline](../docs/architecture/doc_build_system.md)** e **[Desktop UI Stack](../docs/architecture/doc_shell_ui.md)**.

**The absolute zero-trust, high-performance CachyOS-level compiler and package builder for Ermete OS.**

Ermete Forge is the automated CI/CD engine responsible for generating all the custom software, UI components, and configurations that make up Ermete OS. It enforces aggressive compiler optimizations (`-O3`, `-march=x86-64-v3`, `-flto=auto`, `mold` linker) across all custom packages.

Instead of a monolithic script, Ermete Forge distributes each package as a granular **Micro-Container OCI artifact** or RPM.

---

## 🏗️ The Micro-Container OCI Architecture

Every single package or tool has its own independent CI/CD build job producing an isolated `scratch` container image:
- **Zero Monolithic Bloat**: Granular failure isolation and pristine per-package history. If one package fails to build, it doesn't halt the entire Forge, preventing cascading failures.
- **Absolute RPM Encapsulation**: Every system tweak, udev rule, SELinux policy, and GTK application is encapsulated inside a clean `.spec` and `.rpm`. 
- **Tiered Dependency System**: Packages are built in tiers (Tier 0 to Tier 3) ensuring that foundational base packages (like `ermete-base-config`) are built and available before higher-level applications (like `ermete-shell-rs`) attempt to compile.

---

## 📦 Complete Package Registry (`specs/`)

| Spec Directory | RPM Name | Purpose & Architecture |
| :--- | :--- | :--- |
| `ermete-ananicy` | `ermete-ananicy` | Process priority & low-latency scheduling daemon |
| `ermete-base-config` | `ermete-base-config` | Core filesystem hierarchy, RPM Fusion repos, sysusers |
| `ermete-bat` | `ermete-bat` | Syntax-highlighting inspection utility |
| `ermete-bibata` | `ermete-bibata` | Bibata Modern HiDPI cursor theme |
| `ermete-cliphist` | `ermete-cliphist` | Wayland clipboard history daemon |
| `ermete-daemon-rs` | `ermete-daemon-rs` | Pure Rust D-Bus system monitoring daemon |
| `ermete-dart-sass` | `ermete-dart-sass` | Sass compiler for GTK4 stylesheet generation |
| `ermete-desktop-ui` | `ermete-desktop-ui` | Wayland Niri session wrappers and startup scripts |
| `ermete-doctor` | `ermete-doctor` | Rust CLI diagnostics & hardware validation tool |
| `ermete-ide-bootstrap` | `ermete-ide-bootstrap` | Developer toolchain bootstrap & IDE configurations |
| `just` | `just` | Autarchic command runner compiled in Rust with `-O3 -march=x86-64-v3` & mold |
| `kani-verifier` | `kani-verifier` | Kani Rust Formal Verification Engine compiled natively (`kani-driver`, `cargo-kani`) |
| `ermete-matugen` | `ermete-matugen` | Material You dynamic wallpaper color palette generator |
| `ermete-nix-support` | `ermete-nix-support` | Multi-user Nix package manager integration |
| `ermete-selinux` | `ermete-selinux` | Compiled `.pp` SELinux policies for `bootupd` and `scx` |
| `ermete-settings-rs` | `ermete-settings-rs` | Native Rust GTK4 System Settings application |
| `ermete-shell-rs` | `ermete-shell-rs` | Native Rust GTK4 Topbar, Control Center & **Big Tech Login Greeter** |
| `ermete-starship` | `ermete-starship` | Universal cross-shell prompt |
| `ermete-store-rs` | `ermete-store-rs` | Native Rust GTK4 Flatpak & System App Store |
| `ermete-system-config` | `ermete-system-config` | udev rules, presets, `/etc/greetd/config.toml` (Cage Kiosk) |
| `ermete-system-services` | `ermete-system-services` | Systemd service units & timers |
| `ermete-system-tweaks` | `ermete-system-tweaks` | Virtual memory, ZRAM, and I/O latency sysctl tweaks |
| `uki-tools` | `uki-tools` | Unified Kernel Image & Secure Boot toolchain (`sbsigntools` + `systemd-ukify`) |

*(Note: `ermete-kernel` is built via its own standalone pipeline in the `kernel/` directory due to its complexity and `ccache` requirements).*

---

## ⚡ The 100% Rust UI Stack & Kiosk Login Greeter

JavaScript/GJS/NodeJS engines are strictly prohibited in user-space UI for Ermete OS:
1. **Login Greeter**: `ermete-shell-rs --greeter` runs inside a lightweight **`cage`** Wayland Kiosk session (configured by `ermete-system-config`).
2. **Desktop Environment**: `ermete-shell-rs` + `ermete-settings-rs` + `ermete-store-rs` running on the **Niri** scrollable tiling compositor.

---

## 🛡️ Ecosistema di CI/CD Autarchico & Topologia Multi-Stage Build

La Forgia di Ermete OS non scarica mai binari di build da repository esterni di terze parti. I principali strumenti di CI, verifica formale e firma digitale sono scritti/assimilati ed assemblati autonomamente nel **Tier 0**:

### 1. 🛡️ Toolchain Assimilata (Tier 0 Self-Hosted)
- **`kani-verifier`**: Motore di verifica formale e *bounded model checking* per il codice Rust dell'OS. Genera `kani-driver` e `cargo-kani` per validare le proprietà di sicurezza della memoria senza dipendenze da binari esterni.
- **`just`**: Task runner nativo Rust compilato in Forge con `-O3`, `-march=x86-64-v3`, `-fuse-ld=mold` e ThinLTO per l'orchestrazione deterministica dei workflow.
- **`uki-tools`**: Pacchetto di firma e assemblaggio UKI (Unified Kernel Image) che unifica `sbsigntools` (`sbsign`, `sbverify`, `sbattach`, `sbkeysync`, `sbvarsign`) e `systemd-ukify` (`ukify`). Garantisce l'autonomia totale nelle procedure di Secure Boot.

### 🏗️ Workflow Multi-Stage: Il Builder Pesante produce l'OS Leggero

```mermaid
flowchart TD
    subgraph Stage1 ["🏗️ Stage 1: Ermete Forge Heavy Builder (ermete-os-builder)"]
        A1["Compilatori Native (GCC, LLVM/Clang, Rustc)"]
        A2["Tier 0 Autarchic CI Tools (Kani, Just, Sbsign)"]
        A3["Nix-Hermetic Sandbox (Bubblewrap bwrap)"]
        A4["OCI Scratch Micro-Containers per Pacchetto RPM"]
        A1 --> A3
        A2 --> A3
        A3 --> A4
    end

    subgraph Stage2 ["💿 Stage 2: Ermete System Minimal BootC (ermete-os-system)"]
        B1["Fedora BootC Base Image"]
        B2["DNF Bind-Mount Install dagli OCI Scratch"]
        B3["Dracut Initramfs ZSTD Multi-Thread"]
        B4["Epurazione Totale Tool Pesanti Builder (-1.1 GB)"]
        B1 --> B2
        B2 --> B3
        B3 --> B4
    end

    Stage1 -->|RUN --mount=type=bind| Stage2
```

1. **Stage 1 (Heavy Builder)**: Il contenitore `ermete-os-builder` contiene la toolchain pesante completa e genera individualmente gli RPM di Forge in micro-container `scratch`.
2. **Stage 2 (Lightweight Bootable System)**: `system/Containerfile` installa gli RPM in un contesto pulito tramite bind-mount, genera l'initramfs con Dracut ed epura l'intera toolchain di build (rimuovendo GCC, LLVM, sorgenti e header C/Rust per un risparmio netto di -1.1 GB).

---

## 🛠️ Build Pipeline (GitHub Actions)

The `forge` workflow (`.github/workflows/forge-orchestrator.yml`) manages the automated build of all RPMs:
- It processes each tier sequentially.
- Generates the RPM files via `rpmbuild` inside clean container environments.
- Skips GitHub Actions caching (`actions/cache`) for massive RPM artifacts to prevent runner disk exhaustion (`No space left on device` errors).
- Exposes the final compiled RPMs directly to the `system` pipeline, which installs them in the final OCI image without needing GPG signature verification (since they are generated internally).

