# Ermete OS UI Audit Report
**Date:** 2026-07-22
**Target Projects:** `ermete-shell-rs`, `ermete-settings-rs`

This document outlines the identified vulnerabilities, bugs, and architectural flaws across the Ermete OS Rust UI stack.

## 1. Hardcoded Panics and Unsafe Result Unwrapping
The codebase relies heavily on `.unwrap()` and `.expect()` calls, creating significant risks for UI crashes during runtime.

**ermete-shell-rs:**
* **`src/core/system_proxies.rs`:** Widespread use of `state.lock().unwrap()`. If a mutex is poisoned by a panic in a concurrent thread, it will crash the entire proxy backend. Furthermore, DBus interface string conversions (e.g., `try_into().unwrap()` on line 805) are hardcoded to panic.
* **`src/core/dock_config.rs`:** File I/O operations are unsafe. `add_pin("...").expect(...)` and `remove_pin("...").expect(...)` will crash the UI if disk permissions are wrong or the drive is read-only.
* **`src/ui/topbar.rs`:** Missing environment variables or Wayland disconnections will trigger panics. Uses `std::env::var("HOME").unwrap()` and `gtk4::gdk::Display::default().unwrap()`.
* **`src/ui/topbar.rs`:** Inotify watcher initialization uses `notify::recommended_watcher(tx).unwrap()`. If the OS inotify limit is reached, this will instantly crash the shell.

## 2. Broken DBus IPC Connections and Unhandled Timeouts
The Wayland/DBus integration has significant flaws that can cause hanging and unhandled state errors.

* **Missing Timeout Handling (`src/core/battery.rs`):** Async DBus requests like `Connection::system().await?` and `upower.enumerate_devices().await?` propagate errors via `?`, but lack internal timeout logic. If the system `UPower` daemon hangs, it will permanently stall the async tasks.
* **Connection Exhaustion (`src/core/system_proxies.rs`):** `zbus::Connection::session().await` is frequently called inside repeated polling tasks and update loops (e.g., `refresh_mpris`) rather than reusing a static or persistent connection. This leads to file descriptor bloat and timeout bugs under load.
* **Synchronous Widget Blocking (`src/ui/control_center.rs`):** Inline DBus connection polling occurs during widget instantiation, risking momentary freezes in the GTK4 main thread while waiting for IPC responses.

## 3. GTK4/Relm4 UI Vulnerabilities and Logic Dead Ends
Multiple structural flaws in the UI layer impact stability and styling.

* **Missing CSS Watcher Dead End (`src/ui/topbar.rs`):** The app hardcodes CSS paths (`~/.config/ermete-shell/colors.css`). While it implements a string fallback if missing, the `watcher.watch(&path)` call silently drops errors if the directory itself doesn't exist. This silently breaks live-reloading logic, leaving the user with a stale UI.
* **Orphaned Widgets (`src/ui/dock.rs`):** Event handlers aggressively use `popover.connect_closed(|p| p.unparent());`. In complex GTK4 layouts, unparenting a widget without proper cleanup or un-referencing from internal vectors leads to memory leaks or orphaned visual artifacts.
* **Shell Injection Risks (`ermete-settings-rs/src/pages/focus.rs`):** System actions are performed via direct `Command::new("sh")` and `Command::new("systemctl")` calls. This poses a severe logic dead end if the specific shell is missing and prevents safe input sanitization.
