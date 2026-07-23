# Ermete OS Global UI Overhaul & Settings Redesign

## 1. Overview
The goal of this initiative is to elevate the visual standard and user experience of Ermete OS to match or exceed the "premium" feel of tier-1 operating systems (macOS, Windows 11). The primary focus is rewriting `ermete-settings-rs` from raw GTK4 to a scalable Relm4 component architecture, while establishing a global "Ermete Design System" to be shared across all first-party applications.

## 2. Architecture: Relm4 Migration
The current monolithic structure of `ermete-settings-rs` will be refactored into modular components using `relm4`:
- **AppModel**: Central state manager.
- **SidebarComponent**: A macOS-style left navigation pane for categories (Wi-Fi, Bluetooth, Appearance, Desktop, About).
- **PageComponents**: Dynamically loaded content areas on the right side.
- **Interface Segregation**: Complete separation of business logic (DBus, configuration reading) from UI rendering logic.

## 3. Ermete Design System (Global Style)
A unified design language must be enforced:
- **Glassmorphism & Depth**: Surfaces will utilize semi-transparent backgrounds with background-blur (via GTK4/Wayland compositing features when available) and defined box-shadows.
- **Component Primitives**: Creation of standardized widgets (Cards, Toggles, Sliders, Single-line settings rows) that look identical across `ermete-settings-rs`, `ermete-shell-rs`, and future apps.
- **Color Tokens**: CSS will be driven by semantic tokens rather than hardcoded RGBA values, allowing seamless theme switching.

## 4. Typography & Iconography
- **Fonts**: Strict adherence to a highly legible, modern sans-serif typeface (e.g., Inter, Roboto). Font weights will be utilized to establish clear visual hierarchy (bolder headers, muted secondary text).
- **Icons**: Exclusive use of crisp, monochromatic Symbolic Icons for the UI elements to avoid visual clutter and maintain a professional aesthetic.

## 5. Motion & Micro-interactions
- **Tactile Feedback**: Interactive elements (buttons, toggles) must provide immediate visual feedback (e.g., scaling up by 5% on hover, smooth ease-in-out transition on click).
- **Smooth Page Transitions**: Navigating between setting categories will utilize crossfade or sliding transitions instead of abrupt layout shifts.

## 6. Live Theming & Global State Sync
- **IPC Broadcasts**: Changes made in `ermete-settings-rs` (e.g., switching from Dark to Light mode, changing accent colors) will trigger a DBus broadcast.
- **Instant Response**: Other components like `ermete-shell-rs` will listen for these broadcasts and dynamically reload their CSS/state in real-time, eliminating the need for system reboots or service restarts.

## 7. Scope & Phasing
- **Phase 1**: Refactor `ermete-settings-rs` structure to Relm4 and build the foundational CSS/Design System tokens.
- **Phase 2**: Rebuild the settings pages (Wi-Fi, Appearance, Desktop) using the new premium components.
- **Phase 3**: Wire the Live Theming DBus sync between Settings and Shell.
