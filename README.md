# 🦅 Ermeteos OS

**Ermeteos OS** is a modern, immutable, and cloud-native Linux operating system built on top of Fedora and Universal Blue technologies. Engineered strictly for power-users and developers, it discards bloated desktop environments in favor of a lean, ultra-fast, and keyboard-driven Wayland experience.

Driven by an Infrastructure-as-Code (IaC) philosophy, Ermeteos OS is defined entirely by container recipes and built via GitHub Actions. It leverages OCI images to provide unbreakable, atomic system updates.

---

## 🌟 Key Features

### 🛡️ Privacy & Security First
- **Total Anonymity on Clones**: The system dynamically resets `/etc/machine-id` during builds to ensure every instance generates a unique signature on first boot, preventing unintended telemetry tracking.
- **Strict User Boundaries**: The `/etc/skel` profile applies draconian `700/600` permissions.
- **Zero Memory Leaks to Disk**: Core dumps are intentionally disabled at the `systemd` level, protecting sensitive RAM data (passwords, encryption keys) from being written to persistent storage upon application crashes.
- **Firewalled by Default**: `firewalld` is baked in and enabled at boot to aggressively block unsolicited inbound traffic.

### ⚡ High-Performance & Lean Architecture
- **ZRAM Compressed Memory**: Out-of-the-box `zram-generator` configuration handles memory compression dynamically, maximizing multitasking fluidity without prematurely wearing out NVMe/SSD drives via disk swap.
- **Zero-Bloat DNF Builds**: All OS layers are compiled passing `--setopt=install_weak_deps=False`, minimizing the attack surface and keeping the final OCI image surgically thin.
- **Asynchronous Network Boot**: `NetworkManager-wait-online.service` is disabled out-of-the-box, ensuring instantaneous boots.

### 🎨 The "Full-Rust" Graphic Stack
The entire graphical environment and user experience are intentionally built on memory-safe Rust ecosystems:
- **Compositor**: [Niri](https://github.com/YaLTeR/niri) (Scrollable Tiling Wayland Compositor)
- **Status Bar**: [Ironbar](https://github.com/sneexy/ironbar)
- **App Launcher**: [Anyrun](https://github.com/anyrun-org/anyrun)
- **Login Manager / Greeter**: `greetd` strictly confined to `tuigreet`.
- **Core CLI Utilities**: Classical UNIX tools are replaced by modern Rust alternatives (`eza`, `bat`, `fd-find`, `ripgrep`, `bottom`, `nushell`, `starship`).
- **Terminal Emulator**: GPU-accelerated `alacritty` replaces standard emulators.

### 🎮 NVIDIA & Multimedia Unleashed
- **RakuOS Base**: Inherits from the RakuOS Nvidia base image, delivering proprietary drivers and seamless hardware acceleration instantly.
- **RPMFusion Bundled**: Ships with RPMFusion (Free/Non-Free), `ffmpeg`, `libva-utils`, and OBS Studio.

---

## 🚀 Installation & Usage

### Option 1: Clean Install (ISO)
To deploy Ermeteos OS on a bare-metal machine or virtual environment:
1. Navigate to the **Actions** tab of this repository.
2. Select the **Build disk images** workflow.
3. Download the generated `install.iso` artifact.
4. Flash to a USB drive using BalenaEtcher or Rufus, and boot.

### Option 2: Rebase (from an existing bootc/ublue system)
If you are currently running an image-based OS like Bazzite, Bluefin, or Fedora Atomic, you can seamlessly mutate your system into Ermeteos OS by opening a terminal:

```bash
sudo bootc switch ghcr.io/patapem/ermeteos:latest
```
*Reboot your machine when the process finishes.*

---

## 🛠️ Architecture & Development

Ermeteos OS abandons monolithic post-install scripts. Instead, the `Containerfile` sequentially iterates through modular shell scripts in `build_files/recipes/`:

* `01-system-setup.sh`: Base packages, ZRAM, core Rust CLI tools, and Idempotent DNF rules.
* `02-repos-and-codecs.sh`: Third-party repositories and multimedia stacks.
* `03-desktop.sh`: The entire Wayland/Rust shell ecosystem.
* `04-system-config.sh`: Security-first Systemd presets and default dotfiles.
* `05-cleanup.sh`: Cache management and Machine-ID resets for OCI cleanliness.

### Local Build & Testing
A robust `Justfile` is provided for local development. Make sure you have `podman` and `just` installed:
- `just build`: Builds the container image locally.
- `just build-iso`: Generates a bootable ISO.
- `just run-vm-qcow2`: Compiles a QCOW2 image and boots it instantly via QEMU for rapid testing.

---

## 🤝 Acknowledgements

Ermeteos OS stands on the shoulders of giants:
* The [Universal Blue](https://universal-blue.org/) project for the `bootc` frameworks.
* [RakuOS](https://github.com/rakuos/rakuos-base) per the solid NVIDIA base image.
* [Yalter](https://github.com/YaLTeR/niri) for the incredible Niri compositor.
