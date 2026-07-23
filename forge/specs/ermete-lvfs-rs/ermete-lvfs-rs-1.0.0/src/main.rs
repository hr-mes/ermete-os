use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};

mod dbus;
mod firmware;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Ermete LVFS Daemon (Firmware Update Engine)");

    // Export D-Bus interface
    let _conn = zbus::ConnectionBuilder::system()?
        .name("os.ermete.Lvfs")?
        .serve_at("/os/ermete/Lvfs", dbus::LvfsIface)?
        .build()
        .await?;

    info!("D-Bus Interface 'os.ermete.Lvfs' registered.");

    let engine = firmware::FirmwareEngine::new();

    // Main event loop
    loop {
        // Here we could periodically check fwupdmgr in the background
        info!("Polling for firmware updates...");
        
        sleep(Duration::from_secs(86400)).await; // Once a day
    }
}
