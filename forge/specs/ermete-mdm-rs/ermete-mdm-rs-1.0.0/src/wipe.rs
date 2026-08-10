use anyhow::Result;
use tracing::{info, warn};

#[zbus::proxy(
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1",
    interface = "org.freedesktop.systemd1.Manager"
)]
pub trait SystemdManager {
    fn stop_unit(&self, name: &str, mode: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    fn poweroff(&self) -> zbus::Result<()>;
}

pub struct WipeEngine;

impl WipeEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn poll_server(&self) -> Result<String> {
        // Placeholder for remote HTTPS MDM polling
        Ok("OK".into())
    }

    /// Native direct LUKS header key slots erasure via direct async file/block operations
    pub async fn native_cryptsetup_erase(&self, dev_path: &str) -> Result<()> {
        info!("Executing direct cryptographic LUKS header wipe on {}", dev_path);
        if let Ok(mut file) = tokio::fs::OpenOptions::new().write(true).open(dev_path).await {
            use tokio::io::AsyncWriteExt;
            let zeroes = vec![0u8; 1024 * 1024];
            for _ in 0..16 {
                let _ = file.write_all(&zeroes).await;
            }
            let _ = file.flush().await;
        }
        Ok(())
    }

    pub async fn execute_cryptsetup_erase(&self) -> Result<()> {
        warn!("INITIATING CRYPTOGRAPHIC WIPE!");
        
        // Native cryptsetup erase replacement
        let _ = self.native_cryptsetup_erase("/dev/nvme0n1p3").await;

        // Immediately trigger poweroff over D-Bus via systemd Manager proxy
        if let Ok(conn) = zbus::Connection::system().await {
            if let Ok(manager) = SystemdManagerProxy::new(&conn).await {
                let _ = manager.stop_unit("systemd-cryptsetup@luks-root.service", "replace").await;
                let _ = manager.poweroff().await;
            }
        }

        Ok(())
    }
}

