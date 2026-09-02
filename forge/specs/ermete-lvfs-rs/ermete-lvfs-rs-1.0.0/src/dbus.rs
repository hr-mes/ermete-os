use ermete_bus_api::polkit::check_polkit_auth_zbus;
use zbus::interface;
use tracing::{info, error};
use crate::firmware::FirmwareEngine;

pub struct LvfsIface;

#[interface(name = "os.ermete.Lvfs")]
impl LvfsIface {
    /// Apply UEFI/BIOS firmware updates via fwupdmgr. Polkit auth required.
    async fn apply_firmware(
        &self,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to apply firmware.");

        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "os.ermete.lvfs.apply", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;

        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for apply_firmware".into()));
        }
        
        let engine = FirmwareEngine::new();
        
        // Spawn the update process in the background so we don't block the D-Bus loop
        tokio::spawn(async move {
            match engine.check_and_update().await {
                Ok(_) => info!("Firmware update staged successfully in the background."),
                Err(e) => error!("Failed to stage firmware update: {}", e),
            }
        });
        
        Ok("Firmware update process started in the background.".into())
    }
}
