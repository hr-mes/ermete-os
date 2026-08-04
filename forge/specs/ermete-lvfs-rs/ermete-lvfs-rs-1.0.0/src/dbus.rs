use zbus::interface;
use tracing::{info, error};
use crate::firmware::FirmwareEngine;

pub struct LvfsIface;

#[interface(name = "os.ermete.Lvfs")]
impl LvfsIface {
    /// Apply UEFI/BIOS firmware updates via fwupdmgr. Polkit auth required.
    async fn apply_firmware(&self) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to apply firmware.");
        
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
