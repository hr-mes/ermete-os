# Ermete OS: OpenXR Volumetric Compositing Architecture

This document outlines the architectural design for integrating OpenXR into Ermete OS to enable Spatial Computing. By binding `ermete-shell-rs` and `niri` to OpenXR, Ermete OS projects standard GTK Wayland windows into a 3D volumetric AR/VR space.

## Vision: Ermete OS as a Spatial Computing Platform

The goal is to seamlessly transition the 2D Wayland desktop environment into a fully immersive, 3D spatial computing experience without requiring applications to be rewritten. Standard GTK applications will exist as interactive 2D planes within a 3D volumetric environment.

## Core Components

1.  **Wayland Compositor (`niri`)**:
    *   Acts as the primary display server and 2D window manager.
    *   Instead of rendering directly to a physical 2D screen, `niri` renders application surfaces to off-screen buffers (textures).
    *   Exposes these buffers and window metadata (position, size, state) to the OpenXR integration layer.

2.  **Shell and UI (`ermete-shell-rs`)**:
    *   The pure Rust GTK4-based shell continues to manage the system UI (topbar, control center, settings).
    *   UI elements are also rendered to off-screen buffers.
    *   The shell gains spatial awareness, allowing UI elements to be positioned optimally in the 3D space (e.g., a curved system menu floating near the user's hand).

3.  **OpenXR Bridge / Volumetric Compositor**:
    *   A new component (or an extension to `niri`/`ermete-shell-rs`) that acts as an OpenXR application.
    *   Interfaces with the OpenXR runtime (e.g., Monado) to acquire the VR/AR headset's pose, tracking data, and input.
    *   Takes the off-screen buffers produced by `niri` and `ermete-shell-rs` and maps them onto 3D quads or curved surfaces in the virtual environment.
    *   Handles input translation: maps OpenXR controller rays/interactions to standard Wayland pointer/touch events, passing them back to `niri` to be routed to the correct GTK application.

## Data Flow

1.  **Application Rendering**: GTK Wayland app -> `niri` -> Off-screen Texture.
2.  **Spatial Compositing**: OpenXR Bridge reads Texture -> Maps to 3D Quad -> Submits to OpenXR Runtime for display in the headset.
3.  **Input Handling**: OpenXR Controller -> OpenXR Bridge (Raycast intersection with 3D Quad) -> Translated to 2D Wayland Coordinate -> `niri` -> GTK Wayland app.

## Advantages

*   **Zero App Modification**: Existing Wayland/GTK apps run natively in 3D space.
*   **Performance**: Leveraging Rust and zero-copy texture sharing (via DMA-BUF) between Wayland and OpenXR ensures minimal latency, which is critical for VR.
*   **Native Integration**: The entire stack remains strictly within the Ermete OS architecture (Pure Rust Shell, Wayland Compositor).
