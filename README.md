
# 🦅 Ermeteos OS

Ermeteos OS is a modern, immutable, and cloud-native Linux operating system built on top of Fedora and Universal Blue technologies[cite: 36]. Designed for power users, it discards bloated desktop environments in favor of a clean, keyboard-driven Wayland experience.

By utilizing an Infrastructure-as-Code (IaC) approach, Ermeteos OS is entirely defined by container recipes and automatically built via GitHub Actions[cite: 36].

## 🌟 Key Features

* **Immutable & Atomic**: Powered by `bootc` and Universal Blue, system updates are delivered as container images, guaranteeing reliable rollbacks and unbreakable core systems[cite: 36].
* **NVIDIA Ready**: Built on top of the RakuOS Nvidia base image, ensuring proprietary drivers and hardware acceleration are configured right out of the box[cite: 36].
* **Scrollable Tiling**: Features the innovative **Niri** Wayland compositor paired with the Dank Menu System (DMS) for a unique workflow[cite: 36].
* **Multimedia Unleashed**: Pre-configured with RPMFusion (free and nonfree) repositories, bundled with `ffmpeg`, `libva-utils`, and OBS Studio[cite: 36].
* **Power-User Stack**: Includes the `greetd` lightweight login manager, Kitty terminal, and a pre-configured `just` command runner[cite: 36].
* **Homebrew Enabled**: Includes preset Systemd services for seamless Homebrew integration[cite: 36].

---

## 🚀 Installation & Usage

### Option 1: Clean Install (ISO)
If you want to install Ermeteos OS on a fresh machine or virtual machine:
1. Go to the **Actions** tab of this repository.
2. Select the **Build disk images** workflow[cite: 36].
3. Download the generated `install.iso` from the workflow artifacts[cite: 36].
4. Flash it to a USB drive using BalenaEtcher or Rufus, and boot your machine.

### Option 2: Rebase (from an existing bootc/ublue system)
If you are already running an image-based OS like Bazzite, Bluefin, or Fedora Atomic, you can seamlessly mutate your system into Ermeteos OS by opening a terminal and running:

```bash
sudo bootc switch ghcr.io/patapem/ermeteos:latest
```

Reboot your machine when the process finishes.

---

## 🛠️ Architecture & Development

Ermeteos OS abandons monolithic post-install scripts in favor of a modular, highly cacheable container build architecture.

### The Recipe System

The `Containerfile` iterates through independent shell scripts located in `build_files/recipes/`. This separation of concerns makes debugging and forking incredibly easy:

* `01-system-setup.sh`: Base packages and idempotent DNF configurations.


* `02-repos-and-codecs.sh`: Third-party repositories (COPR, RPMFusion).


* `03-desktop.sh`: Wayland compositor and shell installation.


* `04-system-config.sh`: Systemd services and skel defaults.


* `05-cleanup.sh`: Cache removal to minimize image size.



### Local Build & Testing

The repository includes a comprehensive `Justfile` to facilitate local development. Make sure you have `podman` and `just` installed, then run:

* `just build`: Builds the container image locally.


* `just build-iso`: Generates a bootable ISO.


* `just run-vm-qcow2`: Builds a QCOW2 image and immediately boots it via QEMU for rapid testing.



---

## 🤝 Acknowledgements

Ermeteos OS stands on the shoulders of giants. Special thanks to:

* The [Universal Blue](https://universal-blue.org/) project for the `bootc` templates and robust CI/CD frameworks.


* [RakuOS](https://www.google.com/search?q=https://github.com/rakuos/rakuos-base) for the solid NVIDIA base image.


* [Yalter](https://www.google.com/search?q=https://github.com/YaLTeR/niri) for the incredible Niri compositor.


* [AvengeMedia](https://www.google.com/search?q=https://github.com/avengemedia) for the Dank Menu System shell.



```

```
