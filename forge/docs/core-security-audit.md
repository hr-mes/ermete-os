# Ermete OS Phase 3 Technical Debt Audit: System Core & Security

## 1. Transactional Flaws (`ermete-updater-rs`)
- **Missing Rollback Mechanism**: The `UpdateEngine` in `engine.rs` provides no automated rollback handling if an update fails or if a post-update validation fails.
- **Unsafe Fallback to `apply-live`**: In `engine.check_and_apply()`, if the JSON parse of `rpm-ostree status` fails, the updater incorrectly logs a warning and proceeds to execute `rpm-ostree apply-live`. This is a critical transactional flaw that could leave the system in an unstable state if Tier 0 packages were actually updated.

## 2. Security & Polkit Bypass
- **Gatekeeper Sandbox Bypass (`ermete-gatekeeper-rs`)**: The D-Bus interface `os.ermete.Gatekeeper` exposes the `approve_execution(fd_id)` method without any Polkit authentication checks. A local attacker can guess the `fd_id` (which increments sequentially) and authorize the execution of a quarantined malicious binary, bypassing the sandbox entirely.
- **MDM Remote Wipe Privilege Escalation (`ermete-mdm-rs`)**: The `trigger_local_wipe()` method in `dbus.rs` is exposed over the system bus but lacks Polkit checks. Any unprivileged user can invoke this method, triggering a cryptographic erase of the LUKS root partition (`cryptsetup erase`), permanently destroying all data.
- **Overly Permissive SELinux Rules (`ermete-selinux`)**: The `ermete_scx.te` policy explicitly permits `allow kernel_t self:process execmem;`, globally disabling W^X protection in the kernel context. This allows arbitrary code execution in kernel memory, drastically increasing the severity of kernel exploits.
- **Missing Polkit Check in Updater (`ermete-updater-rs`)**: `apply_updates()` in `dbus.rs` specifies in comments that it "requires Polkit authentication," but no such checks are implemented, allowing any user to trigger system updates.

## 3. Core Panics
- **Gatekeeper Hardcoded Unwrap (`ermete-gatekeeper-rs`)**: In `main.rs`, `let path = std::ffi::CString::new(*mount).unwrap();` crashes the `GatekeeperManager` daemon if a mount path ever dynamically resolves with a null byte.
- **Telemetry Panic Vector (`ermete-telemetry-rs`)**: In `github.rs`, the HTTP client initialization explicitly panics with `.expect("Failed to build HTTP client — this should never fail")`. If the TLS backend or system CA certificates are misconfigured, this hyper-critical root daemon will crash instantly.

## 4. Data Leaks (`ermete-telemetry-rs`)
- **Plain-text Credential Leak**: In `github.rs`, `GitHubReporter::report_crash()` reads a plain-text GitHub Personal Access Token directly from the user's home directory (`/home/ermete/.github_token`). This exposes the token to any process that has read access to the daemon's memory, and inappropriately ties a system-wide root service to a user-specific credential.
