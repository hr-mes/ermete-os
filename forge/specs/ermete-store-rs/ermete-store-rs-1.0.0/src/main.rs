use anyhow::Result;
use std::env;
use tracing::info;

mod backend;
mod dbus;
mod ui;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args: Vec<String> = env::args().collect();
    let is_daemon = args.iter().any(|arg| arg == "--daemon");

    if is_daemon {
        info!("Starting Ermete Store in Daemon mode (Zbus Server)...");
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(crate::dbus::server::run_server())?;
    } else {
        info!("Starting Ermete Store UI (GTK4)...");
        crate::ui::window::run_app()?;
    }

    Ok(())
}
