use anyhow::Result;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod systemd_manager;
use systemd_manager::SystemdManager;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("--------------------------------------------------");
    info!("Starting Ermete OS Supervisor (ermete-init-oracle)");
    info!("Role: Systemd State Monitor & Health Recovery");
    info!("--------------------------------------------------");

    // 2. Initialize Systemd Manager (Now using DBus/Systemctl appropriately)
    let manager = SystemdManager::new();

    // 3. Spawn Background Health Audit Loop
    // In a real scenario, this would query DBus signals rather than a polling loop.
    let manager_clone = manager.clone();
    let audit_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = manager_clone.run_health_audit_cycle().await {
                tracing::error!("Health audit cycle failed: {}", e);
            }
        }
    });

    info!("Ermete OS Supervisor is monitoring critical systemd services.");

    // 4. Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT, shutting down Supervisor...");
        }
        res = audit_task => {
            if let Err(e) = res {
                tracing::error!("Audit task panicked: {}", e);
            }
        }
    }

    Ok(())
}
