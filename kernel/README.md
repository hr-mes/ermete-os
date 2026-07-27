<div align="center">
  <h1>⚡ Ermete Kernel (Chimera Kernel)</h1>
  <p><b>The custom, hyper-optimized heart of Ermete OS.</b></p>
</div>

---

## Overview

The `kernel` sub-project of Ermete OS is responsible for compiling the **Chimera Kernel**, a heavily customized and optimized Linux kernel tailored specifically for the Ermete OS immutable environment.

While we heavily leverage the incredible upstream work from the [CachyOS Linux](https://github.com/CachyOS/linux) project (including their CPU schedulers and patches), the Ermete Kernel is built via a custom pipeline to produce a highly encapsulated RPM package (`ermete-kernel.rpm`) that drops its payloads directly into `/lib/modules/` to comply with Ostree and `bootc` immutable image generation standards.

## 🚀 Key Features

* **CachyOS Foundation**: Built on top of the latest CachyOS patches, including the EEVDF and BORE schedulers, BBR3 TCP congestion control, and MGLRU enhancements.
* **Extreme Compiler Optimizations**: Compiled using `gcc` and `ccache` with specific flags tuned for Ermete OS target hardware.
* **Immutable-Ready Architecture**: Unlike traditional kernel packages that install to `/boot`, the Chimera kernel is packaged entirely into `/lib/modules/$KREL/vmlinuz`. This allows the `system` build pipeline to dynamically generate the `initramfs` (using `dracut`) inside the final OCI container image, preserving the `bootc` atomic update mechanism.
* **Automated CI/CD**: The kernel is automatically built via GitHub Actions (`.github/workflows/kernel-build.yml`). The workflow leverages `ccache` to reduce compilation times on subsequent builds, ensuring rapid iteration without exhausting GitHub Runner limits.

## 🛠️ Build Process (`build-kernel-rpm.sh`)

The kernel compilation is fully automated and encapsulated within a single script:

1. **Patching**: CachyOS patches are applied to the pristine Linux source tree.
2. **Configuration**: A bespoke `.config` is injected, enabling required features for Wayland, Niri, Rust UI, and NVIDIA DKMS compatibility.
3. **Compilation**: `make rpm-pkg` is executed with `ccache` enabled, utilizing all available CPU cores.
4. **Repackaging (`ermete-kernel.spec`)**: The resulting RPMs are extracted and repackaged using our custom spec file. This crucial step moves the `vmlinuz` binary out of `/boot` and into `/lib/modules/$KREL/`, which is mandatory for the Ermete OS `dracut` workflow during the final system image assembly.

## 📦 Artifacts

The output of this sub-project is a single RPM file (`ermete-kernel-<version>.rpm`). 
During the main OS build (see the `system` sub-project), this RPM is installed, and a specialized Dockerfile step locates the Chimera kernel within `/lib/modules/` to run `depmod` and `dracut`.

## 🤝 Upstream Credits

This project would not be possible without the foundational work of the **CachyOS** team. We use their patchsets to provide maximum responsiveness and throughput for Ermete OS.

* [CachyOS Linux](https://github.com/CachyOS/linux)
* [BORE Scheduler](https://github.com/firelzrd/bore-scheduler)
