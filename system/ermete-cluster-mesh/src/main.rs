use anyhow::Result;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use zbus::connection::Builder;

mod dbus;
mod discovery;
mod npu_scheduler;
mod pqc_mesh_client;
mod swarm_ipc;
mod swarm_manager;
mod types;

use dbus::ClusterMeshInterface;
use discovery::ZeroConfDiscovery;
use npu_scheduler::NpuScheduler;
use pqc_mesh_client::PqcMeshClient;
use swarm_ipc::SwarmIpcServer;
use swarm_manager::SwarmManager;
use types::NpuCapabilities;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default tracing subscriber failed");

    info!("--------------------------------------------------");
    info!("Starting Ermete OS Cluster Mesh Daemon (ermete-cluster-mesh)");
    info!("Multi-Node Zero-Conf P2P Swarm Computing for NPU Llama 3.2");
    info!("--------------------------------------------------");

    // 2. Load Node Identity & NPU Capabilities
    let local_node_id = format!("node-{}", rand::random::<u16>());
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::fs::read_to_string("/etc/hostname").map(|s| s.trim().to_string()))
        .unwrap_or_else(|_| "ermete-os-node".to_string());

    let local_npu = NpuCapabilities::default();
    info!(
        "Local Node ID: {} ({}) | Active NPU: {} ({:.1} TOPS)",
        local_node_id, hostname, local_npu.device_name, local_npu.tops
    );

    // 3. Initialize PQC Mesh Bus Client
    let pqc_client = PqcMeshClient::new();
    match pqc_client.check_status().await {
        Ok(st) => info!("PQC Mesh Bus integration active: {}", st),
        Err(e) => info!("PQC Mesh Bus not available yet ({}), running in standalone zero-conf mode", e),
    }

    // 4. Initialize NPU Scheduler & Swarm Manager
    let npu_scheduler = Arc::new(NpuScheduler::new(local_node_id.clone(), local_npu.clone()));
    let swarm_manager = Arc::new(SwarmManager::new(
        local_node_id.clone(),
        local_npu.clone(),
        pqc_client,
        npu_scheduler.clone(),
    ));

    // 5. Start Swarm IPC Server (Listening on TCP 51823)
    let ipc_port = 51823;
    let ipc_server = Arc::new(SwarmIpcServer::new(ipc_port));
    ipc_server
        .start_listener(local_node_id.clone(), npu_scheduler)
        .await?;

    // 6. Start Zero-Conf P2P PQC Discovery (UDP Broadcast on 51822)
    let discovery_port = 51822;
    let discovery = Arc::new(ZeroConfDiscovery::new(
        discovery_port,
        ipc_port,
        local_node_id,
        hostname,
        std::env::var("DILITHIUM_PK").expect("CRITICAL: DILITHIUM_PK env var missing").to_string(),
        std::env::var("KYBER_PK").expect("CRITICAL: KYBER_PK env var missing").to_string(),
        std::env::var("X25519_PK").expect("CRITICAL: X25519_PK env var missing").to_string(),
        local_npu,
    ));

    discovery.start(swarm_manager.clone()).await?;

    // 7. Register ZBus DBus Service org.ermete.ClusterMesh
    let dbus_interface = ClusterMeshInterface::new(swarm_manager.clone());
    let _connection = Builder::session()?
        .name("org.ermete.ClusterMesh")?
        .serve_at("/org/ermete/ClusterMesh", dbus_interface)?
        .build()
        .await?;

    info!("DBus service 'org.ermete.ClusterMesh' bound at '/org/ermete/ClusterMesh'");
    info!("Ermete OS Swarm Cluster Mesh is running continuously.");

    // 8. Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT shutdown signal, shutting down Cluster Mesh...");
        }
    }

    Ok(())
}
