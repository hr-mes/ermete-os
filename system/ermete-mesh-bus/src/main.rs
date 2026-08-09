use anyhow::Result;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use zbus::connection::Builder;

mod dbus;
pub mod ipc;
pub mod network;
mod peer;
mod pqc;
pub mod protocol;
mod tunnel;

use dbus::MeshBusInterface;
use peer::PeerManager;
use pqc::PqcEngine;
use tunnel::MeshTunnel;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default tracing subscriber failed");

    info!("--------------------------------------------------");
    info!("Starting Ermete OS PQC Mesh Bus Daemon (ermete-mesh-bus)");
    info!("Level 13 Zero-Trust Post-Quantum WireGuard Evolution");
    info!("--------------------------------------------------");

    // 2. Initialize PQC Cryptographic Engine (ML-KEM-1024 / Dilithium5 / X25519)
    let pqc_engine = PqcEngine::new(None)?;
    let identity = pqc_engine.get_node_identity();

    info!("Local Node Identity: {}", identity.node_id);
    info!("X25519 Public Key: {}", identity.x25519_public_b64);
    info!("ML-KEM-1024 Public Key: {}", identity.kyber_public_b64);
    info!("Dilithium5 Public Key: {}", identity.dilithium_public_b64);

    // 3. Initialize Peer Manager
    let peer_manager = PeerManager::new();

    // 4. Initialize User-Space UDP Mesh Tunnel (listening on 0.0.0.0:51820)
    let tunnel = match MeshTunnel::bind("0.0.0.0:51820", pqc_engine.clone(), peer_manager.clone()).await {
        Ok(t) => Arc::new(t),
        Err(e) => {
            info!("Port 51820 unavailable ({}), trying fallback port 51821...", e);
            Arc::new(MeshTunnel::bind("0.0.0.0:51821", pqc_engine.clone(), peer_manager.clone()).await?)
        }
    };

    // 5. Expose ZBus DBus Interface org.ermete.MeshBus
    let dbus_interface = MeshBusInterface::new(
        pqc_engine.clone(),
        peer_manager.clone(),
        Some(tunnel.clone()),
    );

    let _connection = Builder::system()?
        .name("org.ermete.MeshBus")?
        .serve_at("/org/ermete/MeshBus", dbus_interface)?
        .build()
        .await?;

    info!("DBus service 'org.ermete.MeshBus' bound at path '/org/ermete/MeshBus'");

    // 6. Spawn Async UDP Packet Receiver Loop
    let tunnel_task = tokio::spawn(async move {
        if let Err(err) = tunnel.run_packet_loop().await {
            tracing::error!("MeshTunnel packet loop error: {}", err);
        }
    });

    info!("Ermete OS PQC Mesh Bus is running continuously.");

    // 7. Wait for shutdown signal or tunnel task finish
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT shutdown signal, shutting down PQC Mesh Bus...");
        }
        res = tunnel_task => {
            if let Err(e) = res {
                tracing::error!("Tunnel task joined with error: {}", e);
            }
        }
    }

    Ok(())
}
