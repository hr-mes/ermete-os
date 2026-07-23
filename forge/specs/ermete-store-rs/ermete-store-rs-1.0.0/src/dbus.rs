use zbus::interface;
use tracing::{info, error};
use crate::flatpak::FlatpakManager;
use crate::flathub::FlathubClient;

pub struct StoreIface {
    flathub: FlathubClient,
    flatpak: FlatpakManager,
}

impl StoreIface {
    pub fn new() -> Self {
        Self {
            flathub: FlathubClient::new(),
            flatpak: FlatpakManager::new(),
        }
    }
}

#[interface(name = "os.ermete.Store")]
impl StoreIface {
    /// Search apps via Flathub API
    async fn search_apps(&self, query: String) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus search query: {}", query);
        match self.flathub.search(&query).await {
            Ok(results) => Ok(serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string())),
            Err(e) => {
                error!("Flathub search failed: {}", e);
                Ok("[]".to_string())
            }
        }
    }

    /// Get detailed metadata of a specific app
    async fn get_app_details(&self, app_id: String) -> std::result::Result<String, zbus::fdo::Error> {
        match self.flathub.get_app_details(&app_id).await {
            Ok(details) => Ok(details),
            Err(e) => {
                error!("Failed to get details for {}: {}", app_id, e);
                Ok("{}".to_string())
            }
        }
    }

    /// List all installed flatpak apps as a JSON array
    async fn list_installed(&self) -> std::result::Result<String, zbus::fdo::Error> {
        match self.flatpak.list_installed().await {
            Ok(json) => Ok(json),
            Err(e) => {
                error!("Failed to list installed apps: {}", e);
                Ok("[]".to_string())
            }
        }
    }

    /// Installs a Flatpak app system-wide.
    async fn install_app(&self, app_id: String) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to install app: {}", app_id);
        
        match self.flatpak.install_app(&app_id).await {
            Ok(_) => Ok(format!("Successfully installed {}", app_id)),
            Err(e) => Err(zbus::fdo::Error::Failed(format!("Error: {}", e))),
        }
    }

    /// Uninstalls a Flatpak app system-wide.
    async fn uninstall_app(&self, app_id: String) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to uninstall app: {}", app_id);
        
        match self.flatpak.uninstall_app(&app_id).await {
            Ok(_) => Ok(format!("Successfully uninstalled {}", app_id)),
            Err(e) => Err(zbus::fdo::Error::Failed(format!("Error: {}", e))),
        }
    }
}
