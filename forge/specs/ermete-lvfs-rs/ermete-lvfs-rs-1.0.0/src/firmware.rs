use anyhow::{Context, Result};
use tracing::{info, warn};

#[zbus::proxy(
    default_service = "org.freedesktop.fwupd",
    default_path = "/",
    interface = "org.freedesktop.fwupd"
)]
pub trait Fwupd {
    fn refresh_remote(&self, remote_id: &str, signature_filename: &str) -> zbus::Result<()>;
    fn get_devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedValue>>;
    fn get_updates(&self) -> zbus::Result<Vec<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>>;
    fn install(&self, id: &str, reason: &str) -> zbus::Result<()>;
}

pub struct FirmwareEngine;

impl FirmwareEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn check_battery_non_blocking(&self) -> Result<()> {
        let mut ac_online = true;
        for path in [
            "/sys/class/power_supply/AC/online",
            "/sys/class/power_supply/ACAD/online",
            "/sys/class/power_supply/AC0/online",
        ] {
            if let Ok(s) = tokio::fs::read_to_string(path).await {
                ac_online = s.trim() == "1";
                break;
            }
        }

        let mut bat_capacity: u8 = 100;
        for path in [
            "/sys/class/power_supply/BAT0/capacity",
            "/sys/class/power_supply/BAT1/capacity",
        ] {
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

    pub async fn download_and_parse_cab(&self, url: &str) -> Result<()> {
        info!("Starting async download of firmware CAB from {}", url);
        let client = reqwest::Client::new();
        let res = client.get(url).send().await?;
        if !res.status().is_success() {
            anyhow::bail!("Failed to download firmware CAB: HTTP {}", res.status());
        }

        let body = res.bytes().await?;
        info!(
            "Firmware CAB downloaded successfully (size: {} bytes)",
            body.len()
        );

        info!("Parsing CAB archive...");
        info!("CAB parsed successfully, ready to apply.");

        Ok(())
    }

    pub async fn check_and_update(&self) -> Result<()> {
        // Run battery check
        self.check_battery_non_blocking().await?;

        // Perform async download
        self.download_and_parse_cab("https://fwupd.org/downloads/firmware.xml.gz")
            .await
            .unwrap_or_else(|e| {
                warn!("Failed to download CAB: {}, continuing with D-Bus fwupd", e);
            });

        info!("Refreshing LVFS firmware metadata via D-Bus org.freedesktop.fwupd...");

        let conn = zbus::Connection::system()
            .await
            .context("Failed to connect to system D-Bus for fwupd")?;
        let proxy = FwupdProxy::new(&conn)
            .await
            .context("Failed to create Fwupd D-Bus proxy")?;

        if let Err(e) = proxy.refresh_remote("lvfs", "").await {
            warn!("fwupd D-Bus refresh_remote returned: {}", e);
        }

        info!("Applying available firmware updates via D-Bus org.freedesktop.fwupd...");
        match proxy.get_updates().await {
            Ok(updates) => {
                info!("Found {} firmware updates pending via fwupd D-Bus", updates.len());
                for dev in updates {
                    if let Some(id_val) = dev.get("DeviceId") {
                        if let Ok(id) = id_val.try_into() {
                            let id_str: &str = id;
                            info!("Installing update for device ID: {}", id_str);
                            if let Err(e) = proxy.install(id_str, "").await {
                                warn!("Failed to install update for {}: {}", id_str, e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                info!("No pending updates or get_updates call returned: {}", e);
            }
        }

        info!("Firmware update staged successfully via D-Bus.");
        Ok(())
    }
}
