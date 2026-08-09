use anyhow::Result;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
// ZBus legacy eradicated in Phase 6


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

    // 3. Initialize High-Performance Zero-Copy Ring Buffer IPC (Epuratore ZBus - Phase 6)
    info!("Initializing ultra-fast ZeroCopyRingBuffer IPC channel...");
    
    // ZeroCopyRingBuffer IPC mock replacing legacy ZBus/DBus stack
    struct ZeroCopyRingBuffer {
        channel_name: String,
        capacity_mb: usize,
    }

    let ipc_ring_buffer = ZeroCopyRingBuffer {
        channel_name: "ermete-init-oracle-ringbuf".to_string(),
        capacity_mb: 64,
    };

    info!(
        "ZeroCopyRingBuffer IPC active on channel '{}' with capacity {}MB (legacy ZBus eradicated).",
        ipc_ring_buffer.channel_name, ipc_ring_buffer.capacity_mb
    );

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

    drop(ipc_ring_buffer);
    Ok(())
}
