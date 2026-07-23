use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};

mod engine;
mod dbus;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing (logging)
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Ermete Updater Daemon (Layer 0 & Layer 1 OTA Engine)");

    // Initialize the D-Bus connection and export the interface
    let _conn = zbus::ConnectionBuilder::system()?
        .name("os.ermete.Updater")?
        .serve_at("/os/ermete/Updater", dbus::UpdaterIface)?
        .build()
        .await?;

    info!("D-Bus Interface 'os.ermete.Updater' registered.");

    // Initialize the dual-layer update engine
    let mut update_engine = engine::UpdateEngine::new().await?;

    // Main event loop
    loop {
        info!("Checking for System Updates (bootc / rpm-ostree)...");
        
        match update_engine.check_and_apply().await {
            Ok(status) => {
                info!("Update check completed. Status: {:?}", status);
            }
            Err(e) => {
                error!("Failed to check/apply updates: {:?}", e);
            }
        }

        // Sleep for an hour before checking again
        // In a real scenario, this would be triggered by a Systemd Timer or D-Bus signal.
        sleep(Duration::from_secs(3600)).await;
    }
}
