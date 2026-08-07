# DRM Leasing Configuration for Niri & Ermete AI Daemon

To push Ermete OS hardware efficiency to its absolute limit, we configure Niri to support **DRM Leasing** via the `wp-drm-lease-v1` Wayland protocol. This allows `ermete-ai-daemon` to acquire exclusive control of a portion of the GPU/NPU for AI offloading, bypassing Wayland entirely for zero-copy inference.

## Niri Configuration

To enable DRM leasing for the AI daemon in Niri, add the following to your Niri config file (`~/.config/niri/config.kdl`):

```kdl
// Enable DRM leasing for the wp-drm-lease-v1 protocol
// This allows specific applications (like ermete-ai-daemon) to take direct
// control of a display controller or GPU resource for zero-copy offloading.
environment {
    ENABLE_DRM_LEASE "1"
    WLR_DRM_LEASE "1"
}

// Optionally, you can restrict DRM leasing only to trusted AI processes
window-rule {
    match app-id="ermete-ai-daemon"
    allow-drm-leasing true
}
```

## System Setup

Ensure the correct groups are applied to the daemon user (usually `render` and `video`):

```bash
usermod -aG video,render ermete-daemon
```

With this configuration, the `ermete-ai-daemon` will bypass the compositor when performing heavy Vulkan/NPU inference, granting a 100% zero-copy architecture and reducing CPU load to near 0%.
