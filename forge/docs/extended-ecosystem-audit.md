# Extended Ecosystem Audit Report: Malignant Metastases

This report outlines the hidden technical debt, concurrency flaws, and vulnerabilities mapped across the broader Ermete OS ecosystem (`ermete-store-rs`, `ermete-daemon-rs`, `ermete-recovery`, `ermete-cloud-rs`, and `ermete-doctor`).

## 1. Concurrency/Async Flaws
*   **`ermete-store-rs` (UI Freezes & Tokio Panics):** In `ermete-store-rs-0.1.0/src/ui.rs`, asynchronous futures are executed on the local GTK thread via `glib::spawn_future_local`. The backend functions heavily rely on `tokio::process::Command::new("flatpak")`. If no Tokio runtime context is initialized by the main application, these calls will panic at runtime. Additionally, `glib` futures making long-running system calls without proper worker threads can bottleneck UI responsiveness.
*   **`ermete-recovery` (Deadlocking the Main Thread):** `ermete-recovery-1.0.0/src/main.rs` uses standard, synchronous `std::process::Command::new("rpm-ostree").arg("rollback").output()` inside the GTK `connect_response` dialog callback. This completely freezes the UI for the duration of the system rollback (which can take minutes).
*   **`ermete-doctor` (Synchronous Sleep):** `ermete-doctor-0.1.0/src/main.rs` contains hardcoded `std::thread::sleep(std::time::Duration::from_secs(1))` calls, needlessly blocking execution during network diagnostics.

## 2. OS Integration Vulnerabilities
*   **`ermete-recovery` (Unsafe Reboots & Ignored Failures):** Executes `Command::new("systemctl").arg("reboot").spawn()` natively inside UI callbacks. It fails to verify Polkit privileges or handle permission errors properly, merely catching them silently with `let _ =`.
*   **`ermete-cloud-rs` (Arbitrary Output Execution):** `sync.rs` listens on TCP port 9091 for incoming network clipboards and pipes the unauthenticated network payload directly into `tokio::process::Command::new("wl-copy")` via its `stdin`, lacking input sanitization or size limits.
*   **`ermete-daemon-rs` (Unsafe Shellouts):** `settings.rs` blindly injects properties retrieved from DBus into system commands (`dconf write`, `killall`, `wlsunset`, `swww`, `matugen`, `spd-say`). A malformed or malicious DBus payload could cause unintended system behaviors.
*   **`ermete-doctor` (Destructive Self-Healing):** If a single ping to `1.1.1.1` fails, the doctor blindly issues `nmcli networking off` and `on`, resetting the host's entire network stack.

## 3. Hardcoded Panics
*   **`ermete-cloud-rs`:** `sync.rs` utilizes `expect("Failed to bind UDP")`, `expect("Failed to bind TCP")`, and `unwrap()` for broadcast bindings. If a network interface is unavailable or port 9090/9091 is occupied, the continuity daemon instantly crashes.
*   **`ermete-daemon-rs`:** Pervasive use of `unwrap()` and `expect()` in `bluetooth.rs`, `network.rs`, and `portal_screencast.rs` when parsing untyped DBus dictionaries and object paths. Any deviation from the expected DBus signature will panic the entire daemon.

## 4. State/Memory Issues
*   **`ermete-cloud-rs`:** Maintains an unbounded `Arc<Mutex<HashSet<String>>>` for peer discovery without any eviction or TTL strategy. Long-running sessions will indefinitely leak memory as new DHCP leases cycle on the network.
*   **`ermete-daemon-rs`:** Spawns unstructured, detached `tokio::spawn` loops (e.g., in `gatekeeper_listener.rs`, `qos.rs`) that reference global states without cancellation tokens, leading to orphaned zombie tasks upon service reloads.
*   **`ermete-store-rs`:** In `ui.rs`, async UI closures capture heavily cloned components without structured memory cleanup, risking reference cycle leaks when searching and regenerating application list nodes.
