# 🌋 Ermete Forge — Local Package Build Guide

To build individual custom packages or upstream RPMs locally, use the micro-container OCI build tool:

```bash
./scripts/build_rolling_local.sh <package-name>
```

### 📦 100% Rust UI Stack & Core Custom Packages

Custom Ermete OS components inside `specs/` are built individually into isolated RPM artifacts:

1. **`base-config`** (`specs/ermete-base-config`): System hierarchy, repositories, sysusers
2. **`system-config`** (`specs/ermete-system-config`): udev rules, presets, `/etc/greetd/config.toml` Kiosk
3. **`system-services`** (`specs/ermete-system-services`): Systemd service units and timers
4. **`system-tweaks`** (`specs/ermete-system-tweaks`): Low-latency sysctl and ZRAM configurations
5. **`daemon-rs`** (`specs/ermete-daemon-rs`): Native Rust D-Bus system monitoring daemon
6. **`shell-rs`** (`specs/ermete-shell-rs`): Native Rust GTK4 Topbar, Control Center & Kiosk Login Greeter
7. **`settings-rs`** (`specs/ermete-settings-rs`): Native Rust GTK4 System Settings application
8. **`store-rs`** (`specs/ermete-store-rs`): Native Rust GTK4 App Store & Flatpak manager
9. **`doctor`** (`specs/ermete-doctor`): Native Rust system diagnostics and validation CLI
10. **`ui-agent`** (`specs/ermete-ui-agent`): Zero-latency AI layout adapter daemon

Every package should be built individually using `build_rolling_local.sh <package-name>`. Output RPMs are placed in `~/.rpmbuild/RPMS/` or exported to `/work/output/<package-name>/` when mounted inside a container.
