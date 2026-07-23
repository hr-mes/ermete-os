use anyhow::Result;
use std::future::pending;
use tracing::{info, warn, error};

mod portal;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Ermete XDG Desktop Portal (Privacy & Hardware Indicators)");

    // Export D-Bus interface for XDG Desktop Portal
    let _conn = zbus::ConnectionBuilder::session()?
        .name("org.freedesktop.impl.portal.desktop.ermete")?
        .serve_at("/org/freedesktop/portal/desktop", portal::ScreenCastPortal)?
        .serve_at("/org/freedesktop/portal/desktop", portal::CameraPortal)?
        .serve_at("/org/freedesktop/portal/desktop", portal::LocationPortal)?
        .serve_at("/org/freedesktop/portal/desktop", portal::MicrophonePortal)?
        .build()
        .await?;

    info!("D-Bus Interface 'org.freedesktop.impl.portal.desktop.ermete' registered.");

    // The portal daemon stays alive indefinitely on the session bus
    pending::<()>().await;
    
    Ok(())
}
