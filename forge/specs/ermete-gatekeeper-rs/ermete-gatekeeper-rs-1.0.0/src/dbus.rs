//! The `os.ermete.Gatekeeper` D-Bus interface: exposes user-facing approve/deny/rollback
//! actions for quarantined-file executions, gated by Polkit authorization, plus a
//! generic `request_root_privilege` prompt flow.

use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ermete_bus_api::polkit::check_polkit_auth_zbus;
use zbus::interface;
use zbus::message::Header;
use zbus::object_server::SignalEmitter;

use crate::bcachefs::restore_bcachefs_snapshot_impl;
use crate::fanotify::{respond_and_close, FAN_DENY};
use crate::hypervisor::spawn_microvm_isolated_app;


/// D-Bus object backing the `os.ermete.Gatekeeper` interface; holds the shared fanotify
/// fd and the in-flight quarantined-execution state that `approve_execution` /
/// `deny_execution` / `rollback_snapshot` act on.
pub struct GatekeeperManager {
    /// The process-wide fanotify file descriptor events are responded to on.
    pub fanotify_fd: RawFd,
    /// Maps a pending event's `fd_id` (UUID) to the fanotify event's file descriptor.
    pub pending_events: Arc<tokio::sync::Mutex<HashMap<String, i32>>>, // fd_id -> event_fd
    /// Maps a pending event's `fd_id` to the Bcachefs snapshot path taken for it, if any.
    pub pending_snapshots: Arc<tokio::sync::Mutex<HashMap<String, PathBuf>>>, // fd_id -> snapshot_path
}

impl GatekeeperManager {
    /// Constructs a manager sharing the given fanotify fd and pending-state maps with
    /// the daemon's fanotify event loop.
    pub fn new(
        fanotify_fd: RawFd,
        pending_events: Arc<tokio::sync::Mutex<HashMap<String, i32>>>,
        pending_snapshots: Arc<tokio::sync::Mutex<HashMap<String, PathBuf>>>,
    ) -> Self {
        Self {
            fanotify_fd,
            pending_events,
            pending_snapshots,
        }
    }
}

#[interface(name = "os.ermete.Gatekeeper")]
impl GatekeeperManager {
    async fn approve_execution(
        &self,
        fd_id: String,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<()> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "os.ermete.gatekeeper.approve", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit zbus check failed: {}", e)))?;

        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }

        // Clean up pending snapshot registration on approval
        let _ = self.pending_snapshots.lock().await.remove(&fd_id);

        let event_fd = {
            let mut pending = self.pending_events.lock().await;
            pending.remove(&fd_id)
        };

        if let Some(event_fd) = event_fd {
            let fd_path = format!("/proc/self/fd/{}", event_fd);
            let target_path = tokio::fs::read_link(&fd_path).await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to resolve fd: {}", e)))?;

            // Remove quarantine xattr via stable /proc/self/fd path (TOCTOU-safe) offloaded to blocking pool
            let fd_path_clone = fd_path.clone();
            let _ = tokio::task::spawn_blocking(move || {
                xattr::remove(&fd_path_clone, "user.ermete.quarantine")
            }).await;

            // Spawn inside Level 11 hardware-isolated Micro-VM (crosvm / cloud-hypervisor / firecracker), then DENY original unsandboxed execution
            let sandbox_result = spawn_microvm_isolated_app(Path::new(&target_path)).await;

            match sandbox_result {
                Ok(_child) => {
                    // Micro-VM spawned — DENY original unsandboxed execution
                    respond_and_close(self.fanotify_fd, event_fd, FAN_DENY);
                }
                Err(e) => {
                    let target_str = target_path.to_string_lossy().into_owned();
                    eprintln!("Micro-VM isolation failed for {}: {}. Denying.", target_str, e);
                    respond_and_close(self.fanotify_fd, event_fd, FAN_DENY);
                }
            }
            Ok(())
        } else {
            Err(zbus::fdo::Error::InvalidArgs(format!("No pending event for id {}", fd_id)))
        }
    }

    async fn deny_execution(
        &self,
        fd_id: String,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<()> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "os.ermete.gatekeeper.deny", false)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit zbus check failed: {}", e)))?;

        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for deny_execution".into()));
        }

        let _ = restore_bcachefs_snapshot_impl(&fd_id, &self.pending_snapshots).await;
        let event_fd = {
            let mut pending = self.pending_events.lock().await;
            pending.remove(&fd_id)
        };
        if let Some(event_fd) = event_fd {
            respond_and_close(self.fanotify_fd, event_fd, FAN_DENY);
            Ok(())
        } else {
            Err(zbus::fdo::Error::InvalidArgs(format!("No pending event for id {}", fd_id)))
        }
    }

    async fn rollback_snapshot(
        &self,
        fd_id: String,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<bool> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "os.ermete.gatekeeper.rollback", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit zbus check failed: {}", e)))?;

        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for rollback_snapshot".into()));
        }

        restore_bcachefs_snapshot_impl(&fd_id, &self.pending_snapshots).await
    }

    /// D-Bus signal emitted when a quarantined file's execution is intercepted and
    /// needs the user to approve, deny, or roll back via `approve_execution` /
    /// `deny_execution` / `rollback_snapshot`.
    #[zbus(signal)]
    pub async fn prompt_required(
        signal_ctxt: &SignalEmitter<'_>,
        fd_id: &str,
        app_name: &str,
    ) -> zbus::Result<()>;

    async fn request_root_privilege(
        &self,
        req_id: u64,
        reason: &str,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<()> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::Failed("No sender".into()))?.to_owned();
        let _reason = reason.to_string();
        let conn = conn.clone();

        tokio::spawn(async move {
            let iface_ref = match conn.object_server().interface::<_, GatekeeperManager>("/os/ermete/Gatekeeper").await {
                Ok(iface) => iface,
                Err(e) => { eprintln!("Failed to get iface: {}", e); return; }
            };
            let signal_ctxt = iface_ref.signal_emitter().clone();

            let polkit_status = check_polkit_auth_zbus(&conn, sender.as_str(), "os.ermete.gatekeeper.root", true).await;
            let authorized = polkit_status.unwrap_or(false);

            if authorized {
                let _ = GatekeeperManager::permit(&signal_ctxt, req_id).await;
            } else {
                let _ = GatekeeperManager::deny(&signal_ctxt, req_id).await;
            }
        });

        Ok(())
    }

    /// D-Bus signal emitted when a `request_root_privilege` request is granted.
    #[zbus(signal)]
    pub async fn permit(
        signal_ctxt: &SignalEmitter<'_>,
        req_id: u64,
    ) -> zbus::Result<()>;

    /// D-Bus signal emitted when a `request_root_privilege` request is refused.
    #[zbus(signal)]
    pub async fn deny(
        signal_ctxt: &SignalEmitter<'_>,
        req_id: u64,
    ) -> zbus::Result<()>;
}
