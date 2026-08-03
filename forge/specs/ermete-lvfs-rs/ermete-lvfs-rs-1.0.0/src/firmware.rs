use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::info;

pub struct FirmwareEngine;

impl FirmwareEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn check_and_update(&self) -> Result<()> {
        let ac_online = tokio::fs::read_to_string("/sys/class/power_supply/AC/online")
            .await
            .or_else(|_| tokio::fs::read_to_string("/sys/class/power_supply/ACAD/online"))
            .or_else(|_| tokio::fs::read_to_string("/sys/class/power_supply/AC0/online"))
            .map(|s| s.trim() == "1")
            .unwrap_or(true);

        let bat_capacity: u8 = tokio::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
            .await
            .or_else(|_| tokio::fs::read_to_string("/sys/class/power_supply/BAT1/capacity"))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(100);

        if !ac_online && bat_capacity <= 50 {
            anyhow::bail!("AC power required for firmware update (or battery > 50%)");
        }

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
