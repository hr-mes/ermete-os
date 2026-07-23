use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::info;

pub struct FlatpakManager;

impl FlatpakManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn sync_remotes(&self) -> Result<()> {
        info!("Synchronizing Flatpak remotes...");
        let _ = Command::new("flatpak")
            .arg("update")
            .arg("--appstream")
            .arg("-y")
            .output()
            .await
            .context("Failed to sync flatpak appstream")?;
        
        Ok(())
    }
    
    pub async fn install_app(&self, app_id: &str) -> Result<()> {
        info!("Installing Flatpak app: {}", app_id);
        
        let output = Command::new("flatpak")
            .arg("install")
            .arg("flathub")
            .arg(app_id)
            .arg("-y")
            .arg("--noninteractive")
            .output()
            .await?;
            
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Flatpak installation failed: {}", err);
        }
        
        Ok(())
    }

    pub async fn uninstall_app(&self, app_id: &str) -> Result<()> {
        info!("Uninstalling Flatpak app: {}", app_id);
        
        let output = Command::new("flatpak")
            .arg("uninstall")
            .arg(app_id)
            .arg("-y")
            .arg("--noninteractive")
            .output()
            .await?;
            
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Flatpak uninstallation failed: {}", err);
        }
        
        Ok(())
    }

    /// Returns a JSON string of installed apps
    pub async fn list_installed(&self) -> Result<String> {
        let output = Command::new("flatpak")
            .arg("list")
            .arg("--app")
            .arg("--columns=application,name,version")
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!("Failed to list flatpaks");
        }
        
        let text = String::from_utf8_lossy(&output.stdout);
        
        // Convert the tab-separated output to a basic JSON array of objects
        let mut apps = Vec::new();
        for line in text.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let app = serde_json::json!({
                    "id": parts[0].trim(),
                    "name": parts[1].trim(),
                    "version": parts[2].trim()
                });
                apps.push(app);
            }
        }
        
        Ok(serde_json::to_string(&apps)?)
    }
}
