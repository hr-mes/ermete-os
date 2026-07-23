use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};

mod dbus;
mod wipe;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Ermete MDM Daemon (Anti-Theft & Remote Wipe Engine)");

    // Export D-Bus interface
    let _conn = zbus::ConnectionBuilder::system()?
        .name("os.ermete.Mdm")?
        .serve_at("/os/ermete/Mdm", dbus::MdmIface)?
        .build()
        .await?;

    info!("D-Bus Interface 'os.ermete.Mdm' registered.");

    let mut engine = wipe::WipeEngine::new();

    // Main event loop
    loop {
        // Here we poll the remote MDM server for "Lock" or "Wipe" commands
        info!("Polling MDM server for remote commands...");
        
        // if let Ok(command) = engine.poll_server().await {
        //     if command == "WIPE" {
        //         engine.execute_cryptsetup_erase().await?;
        //     }
        // }
        
        sleep(Duration::from_secs(300)).await;
    }
}
