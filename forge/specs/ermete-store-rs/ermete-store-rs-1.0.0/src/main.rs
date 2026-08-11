#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use anyhow::Result;
use tracing::{info, error};
use std::thread;

mod backend;
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Ermete Store DBus backend daemon...");
    if let Err(e) = backend::dbus::start_dbus_server().await {
        error!("DBus server error: {}", e);
    }
    
    // Mantiene in vita il demone
    std::future::pending::<()>().await;
    Ok(())
}
