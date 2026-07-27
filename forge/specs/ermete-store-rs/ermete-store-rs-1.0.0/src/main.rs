use anyhow::Result;
use std::env;
use tracing::info;

mod backend;
mod ui;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Ermete Store UI (GTK4)...");
    crate::ui::window::run_app()?;

    Ok(())
}
