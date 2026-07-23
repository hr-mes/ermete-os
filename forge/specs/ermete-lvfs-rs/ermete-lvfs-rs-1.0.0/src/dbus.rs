use zbus::interface;
use tracing::info;
use crate::firmware::FirmwareEngine;

pub struct LvfsIface;

#[interface(name = "os.ermete.Lvfs")]
impl LvfsIface {
    /// Apply UEFI/BIOS firmware updates via fwupdmgr. Polkit auth required.
    async fn apply_firmware(&self) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to apply firmware.");
        
        let engine = FirmwareEngine::new();
        
        match engine.check_and_update().await {
            Ok(_) => Ok("Firmware update staged for next reboot.".into()),
            Err(e) => Ok(format!("Error: {}", e)),
        }
    }
}
