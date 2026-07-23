# User-Space Drivers Paradigm in Ermete OS

## Introduction
In our pursuit of extreme system stability, security, and the "Vista da Corvo fino alla Bedrock" architectural vision, Ermete OS is adopting a microkernel-like paradigm for driver management. This involves transitioning from traditional monolithic, kernel-space drivers to user-space Rust daemons and eBPF programs.

## Disabling Monolithic Kernel Modules
To enforce this paradigm, legacy kernel modules must be explicitly disabled to prevent them from claiming devices.
This is achieved by passing blacklisting parameters to the kernel boot arguments via `dracut` configurations or the bootloader:
```
modprobe.blacklist=<module_name>
```
For example, disabling a traditional network driver to let a user-space implementation take over.

## Device Mapping and VFIO
Once the kernel modules are bypassed, the devices (PCIe, USB) are mapped to user-space using frameworks like `vfio-pci` (Virtual Function I/O) and `libusb`.
- **VFIO-PCI**: Allows safe, unprivileged, user-space drivers to access PCIe devices with IOMMU protection. This guarantees that a crashing user-space driver cannot corrupt kernel memory or access unauthorized devices.
- **Libusb**: Provides generic access to USB devices from user-space, removing the need for custom USB kernel drivers.

## Rust Daemons & eBPF
The actual driver logic will be implemented entirely in **Rust** to guarantee memory safety and zero-cost abstractions.
- **Rust User-Space Daemons**: These daemons will interact with devices via `vfio-pci` or `libusb`. They will run unprivileged, monitored by systemd, and can be live-updated (Layer 1 Update) without requiring a system reboot (Layer 0).
- **eBPF Programs**: For high-performance requirements (e.g., networking via XDP), eBPF programs will be utilized to process packets directly within the kernel safely, controlled by user-space Rust supervisors.

## Benefits
- **Security**: Drivers run unprivileged; memory bugs in Rust are statically eliminated. IOMMU prevents DMA attacks.
- **Stability**: A driver crash is a process crash, not a kernel panic. The daemon can be automatically restarted.
- **Live Updatability**: Aligning with Ermete OS's Dual-Layer Architecture, user-space drivers reside in Layer 1 and can be updated and applied LIVE without rebooting the core Layer 0 kernel.
