use std::io::Write;
use anyhow::{anyhow, Result};
use tracing::{error, info, warn};
use std::path::Path;

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

    /// Rilevamento dinamico del block device target tramite /etc/crypttab o /proc/mounts
    pub async fn detect_target_device(&self) -> Result<String> {
        if let Ok(content) = tokio::fs::read_to_string("/etc/crypttab").await {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let dev = parts[1];
                    if Path::new(dev).exists() {
                        return Ok(dev.to_string());
                    }
                }
            }
        }

        if let Ok(content) = tokio::fs::read_to_string("/proc/mounts").await {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == "/" {
                    let dev = parts[0];
                    if Path::new(dev).exists() {
                        return Ok(dev.to_string());
                    }
                }
            }
        }

        Err(anyhow!("Errore Critico: Impossibile rilevare dinamicamente il dispositivo di target per il wipe"))
    }

    /// Rimozione crittografica diretta degli slot chiave LUKS header
    pub async fn native_cryptsetup_erase(&self, dev_path: &str) -> Result<()> {
        info!("Executing direct cryptographic LUKS header wipe on {}", dev_path);
        
        if !Path::new(dev_path).exists() {
            error!("Critical failure: Target device {} does not exist!", dev_path);
            return Err(anyhow!("Errore Critico: Il dispositivo di target wipe '{}' non esiste", dev_path));
        }

        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .open(dev_path)
            .await
            .map_err(|e| anyhow!("Errore Critico: Impossibile aprire il dispositivo di target '{}' per la scrittura: {}", dev_path, e))?;

        let zeroes = vec![0u8; 1024 * 1024];
        for _ in 0..16 {
            file.write_all(&zeroes)
                .await
                .map_err(|e| anyhow!("Errore Critico durante la scrittura dei byte zero su '{}': {}", dev_path, e))?;
        }
        file.flush()
            .await
            .map_err(|e| anyhow!("Errore Critico durante il flush del dispositivo '{}': {}", dev_path, e))?;

        info!("Cryptographic LUKS header wipe completed successfully on {}", dev_path);
        Ok(())
    }

    pub async fn execute_cryptsetup_erase(&self, target_device: Option<&str>) -> Result<()> {
        warn!("INITIATING CRYPTOGRAPHIC WIPE!");
        
        let dev_path = match target_device {
            Some(dev) => dev.to_string(),
            None => self.detect_target_device().await?,
        };

        // Esecuzione erase nativo su device validato
        self.native_cryptsetup_erase(&dev_path).await?;

        // Arresto immediato del sistema tramite D-Bus systemd Manager proxy
        if let Ok(conn) = zbus::Connection::system().await {
            if let Ok(manager) = SystemdManagerProxy::new(&conn).await {
                let _ = manager.stop_unit("systemd-cryptsetup@luks-root.service", "replace").await;
                let _ = manager.poweroff().await;
            }
        }

        Ok(())
    }
}

