#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use anyhow::Result;
use tracing::{info, error};
use std::thread;

mod backend;
mod ui;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting DBus backend...");
    thread::spawn(|| {
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(async {
                if let Err(e) = backend::dbus::start_dbus_server().await {
                    error!("DBus server error: {}", e);
                }
            });
        }
    });

    info!("Starting Ermete Store UI (GTK4)...");
    crate::ui::window::run_app();

    Ok(())
}
