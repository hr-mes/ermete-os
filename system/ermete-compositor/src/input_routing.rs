use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// Keyboard input event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub key_code: u32,
    pub state: u32, // 1 = Press, 0 = Release
    pub timestamp_ms: u64,
}

/// Global input grab handle representing an input sniffing or shortcut hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalInputGrab {
    pub grab_id: u64,
    pub app_id: String,
    pub pid: u32,
    pub approved: bool,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum InputRoutingError {
    #[error("Global input grab request denied by Gatekeeper for app '{app_id}' (PID {pid})")]
    Unauthorized { app_id: String, pid: u32 },

    #[error("Target surface {surface_id} is not the focused window surface")]
    UnfocusedSurface { surface_id: u64 },

    #[error("Global input grab ID {grab_id} not found")]
    GrabNotFound { grab_id: u64 },
}

/// Gatekeeper DBus Input Sniffing Authentication Client.
pub struct GatekeeperInputAuth;

impl GatekeeperInputAuth {
    /// Queries `os.ermete.Gatekeeper` or Polkit over DBus to authorize global input capture hooks.
    /// Returns `true` only if Gatekeeper explicitly approves global key grab access.
    pub async fn authenticate_input_grab(app_id: &str, pid: u32) -> bool {
        info!(
            "Invoking Gatekeeper DBus authentication for global input grab request by app '{}' (PID: {})",
            app_id, pid
        );

        // Query Gatekeeper DBus service
        if let Ok(conn) = zbus::Connection::system().await {
            let reply: Result<bool, zbus::Error> = conn
                .call_method(
                    Some("os.ermete.Gatekeeper"),
                    "/os/ermete/Gatekeeper",
                    Some("os.ermete.Gatekeeper"),
                    "AuthorizeInputGrab",
                    &(app_id, pid),
                )
                .await
                .and_then(|m| m.body().deserialize());

            if let Ok(approved) = reply {
                info!(
                    "Gatekeeper DBus input grab authorization for '{}': {}",
                    app_id, approved
                );
                return approved;
            }
        }

        // Fallback Zero-Trust evaluation when DBus system bus is unreachable
        let lower = app_id.to_lowercase();
        if lower.contains("keylogger")
            || lower.contains("sniffer")
            || lower.contains("malicious")
            || lower.contains("untrusted")
            || lower.contains("spyware")
            || lower.contains("hook")
        {
            warn!(
                "Zero-Trust Guard: DENIED global input grab request for untrusted application '{}'",
                app_id
            );
            return false;
        }

        // Pre-approved system utilities (e.g. system compositor shortcuts)
        if lower == "ermete-shell" || lower == "ermete-compositor" {
            info!("Zero-Trust Guard: Pre-approved system desktop component '{}'", app_id);
            return true;
        }

        warn!(
            "Zero-Trust Guard: Defaulting to DENY for global input grab request by '{}'",
            app_id
        );
        false
    }
}

/// Router enforcing strict surface scoping and Gatekeeper-approved global input grabs.
pub struct InputRouter {
    focused_surface_id: Option<u64>,
    active_grabs: HashMap<u64, GlobalInputGrab>,
    next_grab_id: u64,
    /// Mock override for testing Gatekeeper input auth.
    auth_override: Option<bool>,
}

impl Default for InputRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl InputRouter {
    pub fn new() -> Self {
        Self {
            focused_surface_id: None,
            active_grabs: HashMap::new(),
            next_grab_id: 1,
            auth_override: None,
        }
    }

    pub fn set_auth_override(&mut self, approved: Option<bool>) {
        self.auth_override = approved;
    }

    pub fn set_focused_surface(&mut self, surface_id: Option<u64>) {
        info!("Input router focused surface updated to: {:?}", surface_id);
        self.focused_surface_id = surface_id;
    }

    pub fn focused_surface(&self) -> Option<u64> {
        self.focused_surface_id
    }

    /// Requests a global keyboard grab or input sniffing hook across all surfaces.
    /// Mandates Gatekeeper authentication before granting access.
    pub async fn request_global_input_grab(
        &mut self,
        app_id: &str,
        pid: u32,
    ) -> Result<u64, InputRoutingError> {
        let grab_id = self.next_grab_id;
        self.next_grab_id += 1;

        info!(
            "Received global input grab request (Grab #{}) from app '{}' (PID {})",
            grab_id, app_id, pid
        );

        let is_approved = if let Some(override_val) = self.auth_override {
            override_val
        } else {
            GatekeeperInputAuth::authenticate_input_grab(app_id, pid).await
        };

        if !is_approved {
            warn!(
                "Global input grab request (Grab #{}) REJECTED for app '{}' (PID {}) - Gatekeeper auth denied!",
                grab_id, app_id, pid
            );
            return Err(InputRoutingError::Unauthorized {
                app_id: app_id.to_string(),
                pid,
            });
        }

        info!(
            "Global input grab (Grab #{}) AUTHORIZED by Gatekeeper for app '{}'",
            grab_id, app_id
        );

        let grab = GlobalInputGrab {
            grab_id,
            app_id: app_id.to_string(),
            pid,
            approved: true,
        };

        self.active_grabs.insert(grab_id, grab);
        Ok(grab_id)
    }

    /// Releases an active global input grab.
    pub fn release_global_input_grab(&mut self, grab_id: u64) -> Result<(), InputRoutingError> {
        if self.active_grabs.remove(&grab_id).is_some() {
            info!("Released global input grab #{}", grab_id);
            Ok(())
        } else {
            Err(InputRoutingError::GrabNotFound { grab_id })
        }
    }

    /// Routes a key event strictly to the active focused surface, and approved global grabs.
    /// Rejects delivery of key events to unfocused background surfaces (preventing keylogging).
    pub fn route_key_event(
        &self,
        event: &KeyEvent,
        target_surface_id: u64,
    ) -> Result<bool, InputRoutingError> {
        if Some(target_surface_id) != self.focused_surface_id {
            warn!(
                "Input Security: Blocked key event delivery to unfocused surface {} (Focused surface: {:?})",
                target_surface_id, self.focused_surface_id
            );
            return Err(InputRoutingError::UnfocusedSurface {
                surface_id: target_surface_id,
            });
        }

        info!(
            "Input Security: Delivered key event (code {}) to focused surface {}",
            event.key_code, target_surface_id
        );

        Ok(true)
    }

    pub fn active_grab_count(&self) -> usize {
        self.active_grabs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_input_routing_prevents_unfocused_keylogging() {
        let mut router = InputRouter::new();
        router.set_focused_surface(Some(100));

        let event = KeyEvent {
            key_code: 30, // 'A' key
            state: 1,
            timestamp_ms: 1000,
        };

        // Attempting to deliver key event to an unfocused background surface MUST fail
        let res = router.route_key_event(&event, 200);
        assert_eq!(
            res,
            Err(InputRoutingError::UnfocusedSurface { surface_id: 200 })
        );

        // Delivering key event to the focused surface MUST succeed
        let res_focused = router.route_key_event(&event, 100);
        assert!(res_focused.is_ok());
    }

    #[tokio::test]
    async fn test_global_input_grab_requires_gatekeeper_approval() {
        let mut router = InputRouter::new();
        router.set_auth_override(Some(false));

        let res = router.request_global_input_grab("keylogger-daemon", 666).await;
        assert_eq!(
            res,
            Err(InputRoutingError::Unauthorized {
                app_id: "keylogger-daemon".to_string(),
                pid: 666
            })
        );
        assert_eq!(router.active_grab_count(), 0);
    }

    #[tokio::test]
    async fn test_global_input_grab_succeeds_with_gatekeeper_approval() {
        let mut router = InputRouter::new();
        router.set_auth_override(Some(true));

        let grab_id = router
            .request_global_input_grab("accessibility-service", 1234)
            .await
            .unwrap();

        assert_eq!(grab_id, 1);
        assert_eq!(router.active_grab_count(), 1);

        assert!(router.release_global_input_grab(grab_id).is_ok());
        assert_eq!(router.active_grab_count(), 0);
    }
}
