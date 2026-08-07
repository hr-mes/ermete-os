use anyhow::Result;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use zbus::connection::Builder;

mod dbus;
mod intent;
mod systemd_manager;

use dbus::InitOracleInterface;
use systemd_manager::SystemdManager;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default tracing subscriber failed");

    info!("--------------------------------------------------");
    info!("Starting Ermete OS Init System Oracle (ermete-init-oracle)");
    info!("Pillar 4: AI Autonomous Systemd Orchestration Daemon");
    info!("--------------------------------------------------");

    // 2. Initialize Systemd Manager
    let manager = SystemdManager::new();

    // 3. Expose DBus Interface org.ermete.InitOracle
    let dbus_interface = InitOracleInterface::new(manager.clone());

    // Try binding to system bus, fallback to session bus if system bus fails
    let connection_builder = Builder::system();
    let connection = match connection_builder {
        Ok(b) => match b.name("org.ermete.InitOracle") {
            Ok(b2) => match b2.serve_at("/org/ermete/InitOracle", dbus_interface) {
                Ok(b3) => match b3.build().await {
                    Ok(conn) => {
                        info!("DBus service 'org.ermete.InitOracle' bound on System Bus at path '/org/ermete/InitOracle'");
                        conn
                    }
                    Err(e) => {
                        info!("Failed binding to System Bus ({}), falling back to Session Bus...", e);
                        Builder::session()?
                            .name("org.ermete.InitOracle")?
                            .serve_at("/org/ermete/InitOracle", InitOracleInterface::new(manager.clone()))?
                            .build()
                            .await?
                    }
                },
                Err(_) => {
                    Builder::session()?
                        .name("org.ermete.InitOracle")?
                        .serve_at("/org/ermete/InitOracle", InitOracleInterface::new(manager.clone()))?
                        .build()
                        .await?
                }
            },
            Err(_) => {
                Builder::session()?
                    .name("org.ermete.InitOracle")?
                    .serve_at("/org/ermete/InitOracle", InitOracleInterface::new(manager.clone()))?
                    .build()
                    .await?
            }
        },
        Err(_) => {
            Builder::session()?
                .name("org.ermete.InitOracle")?
                .serve_at("/org/ermete/InitOracle", InitOracleInterface::new(manager.clone()))?
                .build()
                .await?
        }
    };

    info!("DBus connection successfully established.");

    // 4. Spawn Background Health & Fallback Audit Loop
    let manager_clone = manager.clone();
    let audit_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            manager_clone.run_health_audit_cycle().await;
        }
    });

    info!("Ermete OS Init Oracle daemon is running continuous systemd orchestration.");

    // 5. Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT shutdown signal, stopping Init System Oracle daemon...");
        }
        res = audit_task => {
            if let Err(e) = res {
                tracing::error!("Audit task joined with error: {}", e);
            }
        }
    }

    drop(connection);
    Ok(())
}
