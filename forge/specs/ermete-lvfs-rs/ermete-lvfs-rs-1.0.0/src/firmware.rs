use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::info;

pub struct FirmwareEngine;

impl FirmwareEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn check_and_update(&self) -> Result<()> {
        info!("Refreshing LVFS firmware metadata...");
        
        let _ = Command::new("fwupdmgr")
            .arg("refresh")
            .arg("--force")
            .output()
            .await
            .context("Failed to refresh fwupdmgr")?;
            
        info!("Applying available firmware updates...");
        
        // This will stage the updates for the next UEFI boot
        let _ = Command::new("fwupdmgr")
            .arg("update")
            .arg("-y")
            .output()
            .await
            .context("Failed to apply firmware updates")?;
            
        Ok(())
    }
}
