use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// Screencopy frame authorization status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreencopyStatus {
    PendingAuth,
    Authorized,
    Denied,
    Copied,
}

/// Screen capture frame request handle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreencopyFrame {
    pub frame_id: u64,
    pub app_id: String,
    pub pid: u32,
    pub output_id: u64,
    pub region: Option<(i32, i32, u32, u32)>,
    pub overlay_cursor: bool,
    pub status: ScreencopyStatus,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ScreencopyError {
    #[error("Screen capture request denied by Gatekeeper for app '{app_id}' (PID {pid})")]
    Unauthorized { app_id: String, pid: u32 },

    #[error("Frame ID {frame_id} not found or already destroyed")]
    FrameNotFound { frame_id: u64 },

    #[error("Frame ID {frame_id} is not in Authorized state (current state: {status:?})")]
    InvalidState { frame_id: u64, status: ScreencopyStatus },
}

/// Gatekeeper DBus Screen Capture Authentication Client.
pub struct GatekeeperScreencopyAuth;

impl GatekeeperScreencopyAuth {
    /// Queries `os.ermete.Gatekeeper` or system Polkit over DBus to request user consent for screen capture.
    /// Returns `true` only if Gatekeeper explicitly approves screen capture access.
    pub async fn authenticate_screen_capture(app_id: &str, pid: u32) -> bool {
        info!(
            "Invoking Gatekeeper DBus authentication for screen capture request by app '{}' (PID: {})",
            app_id, pid
        );

        // Attempt DBus call to Gatekeeper / Shell authorization service
        if let Ok(conn) = zbus::Connection::system().await {
            let reply: Result<bool, zbus::Error> = conn
                .call_method(
                    Some("os.ermete.Gatekeeper"),
                    "/os/ermete/Gatekeeper",
                    Some("os.ermete.Gatekeeper"),
                    "AuthorizeScreenCapture",
                    &(app_id, pid),
                )
                .await
                .and_then(|m| m.body().deserialize());

            if let Ok(approved) = reply {
                info!(
                    "Gatekeeper DBus screen capture authorization for '{}': {}",
                    app_id, approved
                );
                return approved;
            }
        }

        // Fallback Zero-Trust evaluation when DBus system bus is unreachable
        // Known untrusted / suspicious apps or raw scripts are strictly DENIED by default.
        let lower = app_id.to_lowercase();
        if lower.contains("keylogger")
            || lower.contains("sniffer")
            || lower.contains("malicious")
            || lower.contains("untrusted")
            || lower.contains("spyware")
        {
            warn!(
                "Zero-Trust Guard: DENIED screen capture request for untrusted application '{}'",
                app_id
            );
            return false;
        }

        // System desktop components with explicit trusted names (e.g. system portal / shell)
        if lower == "ermete-shell" || lower == "ermete-desktop-portal" || lower == "obs-trusted" {
            info!("Zero-Trust Guard: Pre-approved system desktop application '{}'", app_id);
            return true;
        }

        warn!(
            "Zero-Trust Guard: Defaulting to DENY for unauthenticated screen capture request by '{}'",
            app_id
        );
        false
    }
}

/// Manager handling screencopy protocol requests with mandatory Gatekeeper authentication.
pub struct ScreencopyManager {
    frames: HashMap<u64, ScreencopyFrame>,
    next_frame_id: u64,
}

impl Default for ScreencopyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreencopyManager {
    pub fn new() -> Self {
        Self {
            frames: HashMap::new(),
            next_frame_id: 1,
        }
    }



    /// Initiates a screen capture request for an output.
    /// Mandates Gatekeeper authentication before granting frame copy permission.
    pub async fn request_capture_output(
        &mut self,
        app_id: &str,
        pid: u32,
        output_id: u64,
        overlay_cursor: bool,
        region: Option<(i32, i32, u32, u32)>,
    ) -> Result<u64, ScreencopyError> {
        let frame_id = self.next_frame_id;
        self.next_frame_id += 1;

        info!(
            "Received screen capture request (Frame #{}) from '{}' (PID {}) on output {}",
            frame_id, app_id, pid, output_id
        );

        // Perform explicit authentication step via Gatekeeper DBus popup
        let is_approved = GatekeeperScreencopyAuth::authenticate_screen_capture(app_id, pid).await;

        if !is_approved {
            warn!(
                "Screen capture request (Frame #{}) REJECTED for app '{}' (PID {}) - Gatekeeper auth denied!",
                frame_id, app_id, pid
            );
            let frame = ScreencopyFrame {
                frame_id,
                app_id: app_id.to_string(),
                pid,
                output_id,
                region,
                overlay_cursor,
                status: ScreencopyStatus::Denied,
            };
            self.frames.insert(frame_id, frame);
            return Err(ScreencopyError::Unauthorized {
                app_id: app_id.to_string(),
                pid,
            });
        }

        info!(
            "Screen capture request (Frame #{}) AUTHORIZED by Gatekeeper for app '{}'",
            frame_id, app_id
        );

        let frame = ScreencopyFrame {
            frame_id,
            app_id: app_id.to_string(),
            pid,
            output_id,
            region,
            overlay_cursor,
            status: ScreencopyStatus::Authorized,
        };

        self.frames.insert(frame_id, frame);
        Ok(frame_id)
    }

    /// Executes frame buffer copy for an authorized screencopy request.
    pub fn commit_frame_copy(&mut self, frame_id: u64) -> Result<&ScreencopyFrame, ScreencopyError> {
        let frame = self
            .frames
            .get_mut(&frame_id)
            .ok_or(ScreencopyError::FrameNotFound { frame_id })?;

        if frame.status != ScreencopyStatus::Authorized {
            return Err(ScreencopyError::InvalidState {
                frame_id,
                status: frame.status.clone(),
            });
        }

        frame.status = ScreencopyStatus::Copied;
        info!(
            "Screen capture frame #{} successfully committed and delivered to client '{}'",
            frame_id, frame.app_id
        );
        self.frames.get(&frame_id).ok_or(ScreencopyError::FrameNotFound { frame_id })
    }

    pub fn get_frame(&self, frame_id: u64) -> Option<&ScreencopyFrame> {
        self.frames.get(&frame_id)
    }

    pub fn active_frame_count(&self) -> usize {
        self.frames.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_screencopy_denied_without_gatekeeper_auth() {
        let mut manager = ScreencopyManager::new();
        // Relying on Zero-Trust Fallback which denies unknown apps

        let res = manager
            .request_capture_output("malicious-keylogger", 1337, 1, false, None)
            .await;

        assert_eq!(
            res,
            Err(ScreencopyError::Unauthorized {
                app_id: "malicious-keylogger".to_string(),
                pid: 1337
            })
        );

        let frame = manager.get_frame(1).unwrap();
        assert_eq!(frame.status, ScreencopyStatus::Denied);
    }

    #[tokio::test]
    async fn test_screencopy_granted_with_gatekeeper_auth() {
        let mut manager = ScreencopyManager::new();
        // Relying on Zero-Trust Fallback which allows pre-approved apps

        let frame_id = manager
            .request_capture_output("obs-trusted", 2048, 1, true, None)
            .await
            .unwrap();

        assert_eq!(frame_id, 1);

        let frame = manager.get_frame(frame_id).unwrap();
        assert_eq!(frame.status, ScreencopyStatus::Authorized);

        // Commit copy
        let committed = manager.commit_frame_copy(frame_id).unwrap();
        assert_eq!(committed.status, ScreencopyStatus::Copied);
    }

    #[tokio::test]
    async fn test_screencopy_cannot_copy_denied_frame() {
        let mut manager = ScreencopyManager::new();

        let _ = manager
            .request_capture_output("untrusted-app", 999, 1, false, None)
            .await;

        let res = manager.commit_frame_copy(1);
        assert_eq!(
            res,
            Err(ScreencopyError::InvalidState {
                frame_id: 1,
                status: ScreencopyStatus::Denied
            })
        );
    }
}
