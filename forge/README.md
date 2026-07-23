# 🌋 Ermete Forge — Private OCI Micro-Container & RPM Forge

**The absolute zero-trust, high-performance CachyOS-level compiler and package builder for Ermete OS.**

Ermete Forge enforces aggressive compiler optimizations (`-O3`, `-march=x86-64-v3`, `-flto=auto`, `mold` linker) across all custom packages and distributes each package as a granular **Micro-Container OCI artifact** (`ghcr.io/patapem/ermete-forge-*`).

---

## 🏗️ The Micro-Container OCI Architecture

Every single package or tool has its own independent CI/CD build job producing an isolated `scratch` container image:
- **Zero Monolithic Bloat**: Granular failure isolation and pristine per-package history.
- **Absolute RPM Encapsulation**: Every system tweak, udev rule, SELinux policy, and GTK application is encapsulated inside a clean `.spec` and `.rpm`.

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
| `ermete-kernel` | `ermete-kernel` | Automated compiler for Chimera Linux Kernel |
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

---

## ⚡ The 100% Rust UI Stack & Kiosk Login Greeter

JavaScript/GJS/NodeJS engines are strictly prohibited in user-space UI:
1. **Login Greeter**: `ermete-shell-rs --greeter` runs inside a lightweight **`cage`** Wayland Kiosk session (`ermete-system-config`).
2. **Desktop Environment**: `ermete-shell-rs` + `ermete-settings-rs` + `ermete-store-rs` running on **Niri** scrollable tiling compositor.
