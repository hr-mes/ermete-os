use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};

mod dbus;
mod flatpak;
mod flathub;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Ermete Store Daemon (Universal App Store Engine)");

    // Export D-Bus interface
    let _conn = zbus::ConnectionBuilder::system()?
        .name("os.ermete.Store")?
        .serve_at("/os/ermete/Store", dbus::StoreIface::new())?
        .build()
        .await?;

    info!("D-Bus Interface 'os.ermete.Store' registered.");

    let manager = flatpak::FlatpakManager::new();
    
    // Perform initial sync on startup
    if let Err(e) = manager.sync_remotes().await {
        error!("Failed initial Flatpak sync: {:?}", e);
    }

    // Main event loop
    loop {
        // Here we could periodically check for app updates in the background
        info!("Polling for app updates...");
        
        sleep(Duration::from_secs(3600)).await;
    }
}
