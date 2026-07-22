use zbus::interface;
use tracing::info;
use crate::wipe::WipeEngine;

pub struct MdmIface;

#[interface(name = "os.ermete.Mdm")]
impl MdmIface {
    /// Manually trigger a local device wipe (e.g. from the UI before giving PC away)
    async fn trigger_local_wipe(
        &self,
        #[zbus(header)] hdr: zbus::MessageHeader<'_>,
        #[zbus(connection)] _conn: &zbus::Connection,
    ) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to trigger LOCAL WIPE.");

        let sender = hdr.sender().ok_or(zbus::fdo::Error::Failed("No sender".into()))?;
        let status = std::process::Command::new("pkcheck")
            .arg("--system-bus-name")
            .arg(sender.as_str())
            .arg("--action-id")
            .arg("os.ermete.mdm.wipe")
            .status()
            .map_err(|e| zbus::fdo::Error::Failed(format!("pkcheck failed: {}", e)))?;
            
        if !status.success() {
            return Err(zbus::fdo::Error::Failed("Polkit authorization failed".into()));
        }
        
        let engine = WipeEngine::new();
        
        // This is extremely dangerous, requires Polkit auth
        match engine.execute_cryptsetup_erase().await {
            Ok(_) => Ok("Wipe initiated. System halting.".into()),
            Err(e) => Ok(format!("Error: {}", e)),
        }
    }
}
