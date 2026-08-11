use anyhow::Result;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use ermete_bus_api::shm_ring::ZeroCopyRingBuffer;

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
    info!("Starting Ermete OS Init System Oracle (ermete-init-oracle)");
    info!("Pillar 4: AI Autonomous Systemd Orchestration Daemon");
    info!("--------------------------------------------------");

    // 2. Initialize Systemd Manager
    let manager = SystemdManager::new();

    // 3. Initialize High-Performance Zero-Copy Ring Buffer IPC (Epuratore ZBus - Phase 6)
    info!("Initializing ultra-fast ZeroCopyRingBuffer IPC channel...");

    let ipc_ring_buffer = ZeroCopyRingBuffer::create_named("ermete-init-oracle-ringbuf", 64 * 1024 * 1024)
        .or_else(|_| ZeroCopyRingBuffer::create_anonymous("ermete-init-oracle-ringbuf", 64 * 1024 * 1024))
        .ok();

    info!(
        "ZeroCopyRingBuffer IPC active on channel 'ermete-init-oracle-ringbuf' with capacity 64MB (legacy ZBus eradicated)."
    );


    // 4. Spawn Background Health & Fallback Audit Loop
    let manager_clone = manager.clone();
    let audit_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = manager_clone.run_health_audit_cycle().await {
                tracing::error!("Health audit cycle failed: {}", e);
            }
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
