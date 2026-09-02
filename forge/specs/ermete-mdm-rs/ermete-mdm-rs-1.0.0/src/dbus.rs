//! D-Bus interface plumbing for the `os.ermete.Mdm.trigger_local_wipe` method (polkit
//! gate from `ermete_bus_api::polkit`).

use ermete_bus_api::polkit::check_polkit_auth_zbus;
use zbus::interface;
use tracing::info;
use crate::wipe::WipeEngine;

/// A second `os.ermete.Mdm` D-Bus interface implementation exposing only the
/// destructive local-wipe method. Note: `main.rs` builds and serves its own
/// `MdmDBusInterface` (with `apply_policy`) at object path `/os/ermete/Mdm`
/// under the same interface name; `main.rs` never constructs or serves this
/// `MdmIface` type, so `trigger_local_wipe` as defined here is currently dead
/// code / not wired into the running daemon.
pub struct MdmIface;

#[interface(name = "os.ermete.Mdm")]
impl MdmIface {
    /// D-Bus method: manually triggers a local device wipe (e.g. from the UI
    /// before giving the PC away). Requires Polkit authorization for the
    /// `os.ermete.mdm.wipe` action, then delegates to
    /// [`WipeEngine::execute_cryptsetup_erase`], which overwrites the LUKS
    /// header of the detected root device and powers off the system.
    ///
    /// # Errors
    /// Returns `AccessDenied` if there's no D-Bus sender or the Polkit check
    /// fails/denies. Wipe-engine failures are *not* surfaced as a D-Bus error —
    /// they're folded into an `Ok("Error: ...")` string response instead.
    async fn trigger_local_wipe(
        &self,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to trigger LOCAL WIPE.");

        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "os.ermete.mdm.wipe", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit authorization check failed: {}", e)))?;
            
        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }
        
        let engine = WipeEngine::new();
        
        // This is extremely dangerous, requires Polkit auth
        match engine.execute_cryptsetup_erase(None).await {
            Ok(_) => Ok("Wipe initiated. System halting.".into()),
            Err(e) => Ok(format!("Error: {}", e)),
        }
    }
}
