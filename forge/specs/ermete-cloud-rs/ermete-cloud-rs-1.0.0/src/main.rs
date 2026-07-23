use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};

mod dbus;
mod sync;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Ermete Cloud Daemon (Universal Clipboard & P2P Engine)");

    let sync_engine = std::sync::Arc::new(sync::SyncEngine::new());
    
    // Export D-Bus interface
    let _conn = zbus::ConnectionBuilder::system()?
        .name("os.ermete.Cloud")?
        .serve_at("/os/ermete/Cloud", dbus::CloudIface { engine: sync_engine.clone() })?
        .build()
        .await?;

    info!("D-Bus Interface 'os.ermete.Cloud' registered.");

    
    // Start local mDNS discovery loop (placeholder)
    sync_engine.start_discovery().await?;

    // Main event loop
    loop {
        // Here we keep the daemon alive to listen for P2P connection requests
        sleep(Duration::from_secs(3600)).await;
    }
}
