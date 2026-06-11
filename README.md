<div align="center">
  <h1>🦅 Ermete OS</h1>
  <p><b>An uncompromising, cloud-native, atomic Linux distribution engineered for power-users.</b></p>
</div>

---

**Ermete OS** is a hyper-optimized, immutable operating system built on top of Fedora and Universal Blue (`bootc`) technologies. It discards bloated desktop environments, replacing them with a surgically thin, keyboard-driven Wayland experience written entirely in Rust.

Driven by an absolute **Infrastructure-as-Code (IaC)** philosophy, Ermete OS is defined entirely by OCI container recipes and built via GitHub Actions. It guarantees unbreakable atomic updates, zero system entropy, and uncompromising privacy.

## 🌟 The Enterprise Manifesto

1. **Zero-Entropy**: The root filesystem is immutable. No system degradation over time.
2. **Zero-Bloat**: Only CLI tools and core infrastructure exist on the host. Every GUI application is strictly sandboxed via Flatpak.
3. **100% Cryptographically Verified Supply Chain**: To ensure reproducible and unbreakable builds, the OS relies strictly on signed Fedora/RPMFusion repositories. Third-party opaque binaries are strictly prohibited; external dependencies are either natively compiled from source (via Rust `Cargo`) or cryptographically verified against pinned SHA256 checksums during the OCI build.
4. **Cryptographic Image Signatures (Sigstore/Cosign)**: Every deployment image is signed via Keyless OIDC. Client machines automatically verify the cryptographic authenticity of the OS before applying any update, mathematically preventing downgrade attacks or man-in-the-middle container injections.

---

## 🛡️ Paranoid Privacy & Security Fortress
Ermete OS implements extreme enterprise-grade defaults to protect user data and telemetry.

### 🎯 Threat Model Definition
To provide absolute clarity on our security posture, Ermete OS is engineered to mitigate the following specific threat vectors:
- **Network Surveillance & ISP Tracking**: Neutralized via system-wide DNS-over-TLS (DoT) and physical MAC Address Randomization.
- **Remote Exploitation & Port Scanning**: Neutralized via a Zero-Trust `drop` zone Firewalld policy applied by default.
- **Malicious/Compromised GUI Software**: Neutralized via draconian Flatpak sandboxing (X11 sockets permanently disabled, `~/.home` directory access denied, forcing XDG portals).
- **Post-Exploitation Data Leaks**: Neutralized by disabling systemd coredumps, preventing RAM secrets (passwords, encryption keys) from being flushed to unencrypted disk sectors upon application crashes.
- **Local Privilege Escalation**: Mitigated by aggressive kernel hardening (`kptr_restrict`, unprivileged BPF disabled) and strict SELinux MAC enforcement.
> **Note on Physical Access (Evil Maid Attacks)**: Protection against physical theft or unauthorized local booting requires the user to enable Secure Boot and TPM 2.0 backed LUKS Full Disk Encryption (FDE) during the initial ISO installation.

### 🏰 Architectural Hardening
- **Zero-Trust Network (Firewalld Drop Zone)**: The firewall defaults to `drop` out-of-the-box, ensuring your machine is completely invisible to port scans and unauthorized LAN requests.
- **DNS-over-TLS (DoT)**: Enforced system-wide via `systemd-resolved` to prevent ISP tracking.
- **MAC Address Randomization**: Automatically anonymizes your hardware signature across both Wi-Fi and Ethernet connections.
- **Global Flatpak Hardening**: Flatpaks are globally stripped of X11 socket support (`--nosocket=x11`) and blocked from directly accessing the home directory (`--nofilesystem=home`). Applications must exclusively use XDG Desktop Portals to interact with user files.
- **Total Clone Anonymity**: The `/etc/machine-id` is erased during OCI builds. Every instance generates a unique signature only upon first boot, preventing container telemetry tracking.
- **Zero Memory Leaks to Disk**: Core dumps are intentionally disabled at the `systemd` level. Your RAM data (passwords, encryption keys) will never be written to unencrypted persistent storage upon application crashes.
- **Strict User Boundaries & SELinux**: The `/etc/skel` profile applies draconian `700/600` permissions. SELinux MAC policies are strictly enforced and relabeled during the container build to prevent unauthorized access.

---

## ⚡ Extreme Performance & Architecture
- **ZRAM Compressed Memory**: Out-of-the-box `zram-generator` configuration uses **ZSTD compression** and allocates up to 100% of RAM dynamically (`vm.swappiness=150`), maximizing multitasking fluidity without prematurely wearing out NVMe/SSD drives via disk swap.
- **OOMD Wayland Protection**: `systemd-oomd` is aggressively tuned (90% limit / 5 seconds threshold) to surgically kill memory-hogging processes *before* the Wayland session freezes, guaranteeing total GUI stability during heavy compilations.
- **TCP BBR**: The Linux kernel is tuned to use Google's BBR congestion control algorithm, minimizing latency and maximizing network throughput.
- **Silent & Lightning Boot**: `NetworkManager-wait-online.service` is disabled, Dracut is debloated from legacy modules (floppy, pcspkr), and kernel arguments are tuned for a completely silent, high-speed boot process.
- **Silent Immutable Updates**: A background `bootc-fetch-apply.timer` silently downloads the latest OCI image once a week. The system updates atomically on your next reboot without any manual intervention.
- **Greenboot Auto-Repair**: Systemd health checks continuously monitor Wayland (`greetd`) and Network services. If a critical failure occurs, the OS automatically rolls back to the previous working image.
- **Hardware Maintenance**: Automatic `fstrim` timers and `fwupd` services keep your SSDs and firmware optimized silently in the background.

---

## 🎨 The "Full-Rust" Wayland Stack
The graphical layer abandons monolithic desktops in favor of individual, hyper-fast components built in memory-safe Rust. All components are beautifully themed out-of-the-box using the **Catppuccin Mocha** palette, **Inter** UI fonts, and **JetBrains Mono**.

- **Compositor**: [Niri](https://github.com/YaLTeR/niri) (Scrollable Tiling Wayland Compositor).
- **Status Bar**: [Ironbar](https://github.com/JakeStanger/ironbar) (Compiled natively, featuring custom CSS, FontAwesome icons, and UPower integration).
- **App Launcher**: [Anyrun](https://github.com/anyrun-org/anyrun) (Compiled natively, styled as a modern floating overlay).
- **Login Greeter**: `tuigreet` (Terminal-based greeter with password masking).
- **Terminal**: `Alacritty` (GPU-accelerated, zero-latency emulator).
- **Functional Package Manager**: **Nix** is installed out-of-the-box for purely functional, immutable, and reproducible CLI development tools.
- **Core Utilities**: Modern Rust replacements for UNIX tools (`eza`, `bat`, `fd-find`, `ripgrep`, `bottom`, `nushell`, `starship`).
- **Terminal IDE**: **Neovim** is pre-configured with **LazyVim** (Catppuccin Mocha theme), offering a complete, lightning-fast Rust development environment (LSP, autocomplete) straight out of the box.

---

## 💎 Seamless Desktop Integration
Despite being a power-user terminal-centric OS, Ermete OS integrates flawlessly with modern Wayland standards:
- **Native Noise Suppression**: The PipeWire stack runs **EasyEffects** silently in the background upon login, automatically providing studio-grade echo cancellation and microphone noise reduction.
- **XDG Desktop Portals**: Native `xdg-desktop-portal-gnome` and D-Bus activation ensure screensharing (e.g., OBS Studio, Discord) and GUI file pickers work out of the box.
- **Polkit GUI Escalation**: `lxqt-policykit-agent` runs in the background, allowing graphical apps (like Virt-Manager) to seamlessly prompt for root passwords.
- **Gnome Keyring**: A fully integrated keyring daemon securely manages your SSH keys, OAuth tokens, and browser passwords, automatically unlocking upon login.
- **Tray Applets**: Lightweight `nm-applet` and `blueman` sit quietly in the Ironbar tray for quick, convenient Wi-Fi and Bluetooth management.

---

## 📦 Automated Flatpak Ecosystem (First-Boot)
To preserve host purity, **no graphical software is installed via system packages**.
Upon your very first boot and network connection, Ermete OS silently provisions your user-space via a dedicated `oneshot` systemd service. It globally enforces the **Adwaita Dark** GTK theme and **Papirus** icons across all sandboxes, and automatically installs the "Power-User Trinity" and essential multimedia tools:

1. **Alacritty** (Your primary interface)
2. **Flatseal** (For surgical manipulation of Flatpak permissions)
3. **Warehouse** (For GUI-based Flatpak management, cache clearing, and snapshots)
4. **Firefox** (Your portal to the web and the Flathub store)
5. **Nautilus, MPV, OBS Studio** (For file management and native GPU-accelerated multimedia)

*All Flatpaks leverage the proprietary NVIDIA drivers and RPMFusion codecs seamlessly injected into the base image.*

---

## 🚀 Installation & Usage

### Option 1: Clean Install (ISO)
To deploy Ermete OS on a bare-metal machine or virtual environment:
1. Navigate to the **Actions** tab of this repository.
2. Select the **Build disk images** workflow and download the `install.iso` artifact.
3. Flash to a USB drive using BalenaEtcher or Rufus, and boot.

### Option 2: Rebase (from an existing bootc system)
If you are currently running an image-based OS like Bazzite or Fedora Atomic, seamlessly mutate your system by opening a terminal:
```bash
sudo bootc switch ghcr.io/patapem/ermete-os:latest
```
*Reboot your machine when the process finishes.*

---

## 🛠️ Modifying the OS (IaC)
Ermete OS abandons post-install scripts. To modify the OS, simply edit the `Containerfile` or the modular bash recipes in `build_files/recipes/`:

* `01-system-setup.sh`: Base packages, ZRAM, core Rust CLI tools, Nix Package Manager, and Idempotent DNF rules.
* `02-repos-and-codecs.sh`: RPMFusion repositories and host-level multimedia codecs.
* `03-desktop.sh`: Native Rust compilations for the Wayland UI, plus typography and icon themes.
* `04-system-config.sh`: Security presets (Firewalld Drop Zone), DNS-over-TLS, MAC randomization, OOMD protection, and dotfiles distribution.
* `04b-private-optimizations.sh`: Global Flatpak Hardening, Greenboot auto-repair, silent boot kargs, and the First-Boot Flatpak provisioner (including EasyEffects).
* `04c-kernel-tuning.sh`: Aggressive kernel optimization, TCP BBR, ZSTD ZRAM, and Dracut debloating.
* `05-cleanup.sh`: Cache management, Machine-ID resets, and SELinux relabeling for OCI cleanliness.

### Local Build & Testing
A robust `Justfile` is provided for local development. Make sure you have `podman` and `just` installed:
- `just build`: Builds the container image locally.
- `just build-iso`: Generates a bootable ISO.
- `just run-vm-qcow2`: Compiles a QCOW2 image and boots it instantly via QEMU for rapid testing.

---

## 🤝 Acknowledgements
* The [Universal Blue](https://universal-blue.org/) project for the `bootc` frameworks.
* [RakuOS](https://github.com/rakuos/rakuos-base) for the initial inspiration of the NVIDIA base image.
* [Yalter](https://github.com/YaLTeR/niri) for the incredible Niri compositor.
