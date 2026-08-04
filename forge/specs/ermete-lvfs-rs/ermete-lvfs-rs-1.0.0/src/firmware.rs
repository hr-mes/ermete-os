use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn};

pub struct FirmwareEngine;

impl FirmwareEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn check_battery_non_blocking(&self) -> Result<()> {
        let mut ac_online = true;
        for path in ["/sys/class/power_supply/AC/online", "/sys/class/power_supply/ACAD/online", "/sys/class/power_supply/AC0/online"] {
            if let Ok(s) = tokio::fs::read_to_string(path).await {
                ac_online = s.trim() == "1";
                break;
            }
        }

        let mut bat_capacity: u8 = 100;
        for path in ["/sys/class/power_supply/BAT0/capacity", "/sys/class/power_supply/BAT1/capacity"] {
            if let Ok(s) = tokio::fs::read_to_string(path).await {
                if let Ok(val) = s.trim().parse() {
                    bat_capacity = val;
                    break;
                }
            }
        }

        if !ac_online && bat_capacity <= 50 {
            anyhow::bail!("AC power required for firmware update (or battery > 50%)");
        }

        Ok(())
    }

    pub async fn download_and_parse_cab_mock(&self, url: &str) -> Result<()> {
        info!("Starting async download of firmware CAB from {}", url);
        // Mock download with reqwest
        let client = reqwest::Client::new();
        let res = client.get(url).send().await?;
        if !res.status().is_success() {
            anyhow::bail!("Failed to download firmware CAB: HTTP {}", res.status());
        }
        
        let _body = res.bytes().await?;
        info!("Firmware CAB downloaded successfully (mock size: {} bytes)", _body.len());
        
        info!("Parsing CAB archive...");
        // Mock parsing logic
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        info!("CAB parsed successfully, ready to apply.");

        Ok(())
    }

    pub async fn check_and_update(&self) -> Result<()> {
        // Run battery check
        self.check_battery_non_blocking().await?;

        // Perform async download (mocked)
        self.download_and_parse_cab_mock("https://fwupd.org/downloads/firmware.xml.gz").await.unwrap_or_else(|e| {
            warn!("Failed to mock download CAB: {}, continuing with fwupdmgr", e);
        });

        info!("Refreshing LVFS firmware metadata...");
        
        // Use tokio::process::Command to avoid blocking
        let mut child = Command::new("fwupdmgr")
            .arg("refresh")
            .arg("--force")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn fwupdmgr refresh")?;
            
        let status = child.wait().await?;
        if !status.success() {
            warn!("fwupdmgr refresh returned non-zero status: {}", status);
        }
            
        info!("Applying available firmware updates in the background...");
        
        // Spawn and detach update process so we don't wait if not necessary,
        // but here we wait for the staging to complete (since we are in an async task anyway)
        let mut update_child = Command::new("fwupdmgr")
            .arg("update")
            .arg("-y")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn fwupdmgr update")?;
            
        let status = update_child.wait().await?;
        if !status.success() {
            anyhow::bail!("fwupdmgr update failed with status: {}", status);
        }
            
        info!("Firmware update staged successfully.");
        Ok(())
    }
}
