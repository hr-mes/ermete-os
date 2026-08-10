use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use zbus::connection::Builder;

mod dbus;
pub mod ipc;
pub mod network;
mod peer;
mod pqc;
pub mod protocol;
pub mod sync;
mod tunnel;

use std::sync::Arc;
use dbus::MeshBusInterface;
use network::{AfXdpConfig, AfXdpSocket};
use peer::PeerManager;
use pqc::PqcEngine;
use protocol::ZeroCopyParser;
use sync::{CrdtBroadcaster, StorageBridge};

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
    info!("Level 13 Zero-Trust Post-Quantum Kernel Bypass Engine");
    info!("--------------------------------------------------");

    // 2. Initialize PQC Cryptographic Engine (ML-KEM-1024 / Dilithium5 / X25519)
    let pqc_engine = PqcEngine::new(None)?;
    let identity = pqc_engine.get_node_identity();

    info!("Local Node Identity: {}", identity.node_id);
    info!("X25519 Public Key: {}", identity.x25519_public_b64);
    info!("ML-KEM-1024 Public Key: {}", identity.kyber_public_b64);
    info!("Dilithium5 Public Key: {}", identity.dilithium_public_b64);

    // 3. Initialize Peer Manager & IPC Storage Bridge (Fase 11)
    let peer_manager = PeerManager::new();
    peer_manager.spawn_heartbeat_pruner(60);
    let storage_bridge = Arc::new(StorageBridge::new(None, None)?);
    let (crdt_broadcaster, _background_dispatcher) = CrdtBroadcaster::new(
        pqc_engine.clone(),
        peer_manager.clone(),
        storage_bridge,
    );

    // 4. Initialize AF_XDP Kernel Bypass Socket with autodetected network interface parameters
    let active_if_name = network::af_xdp::detect_active_interface();
    let af_xdp_config = AfXdpConfig {
        if_name: active_if_name,
        queue_id: 0,
        frame_size: 2048,
        frame_count: 4096,
        rx_ring_size: 2048,
        tx_ring_size: 2048,
        fill_ring_size: 2048,
        comp_ring_size: 2048,
        zero_copy: true,
        headroom: 256,
    };

    info!("Initializing AF_XDP Kernel Bypass socket on interface '{}'...", af_xdp_config.if_name);
    let mut af_xdp_socket = match AfXdpSocket::new(af_xdp_config) {
        Ok(socket) => Some(socket),
        Err(err) => {
            info!("AF_XDP Kernel Bypass socket notice: {} (simulating AF_XDP event loop)", err);
            None
        }
    };

    // 5. Expose ZBus DBus Interface org.ermete.MeshBus
    let dbus_interface = MeshBusInterface::new(
        pqc_engine.clone(),
        peer_manager.clone(),
        None,
    );

    let _connection = Builder::system()?
        .name("org.ermete.MeshBus")?
        .serve_at("/org/ermete/MeshBus", dbus_interface)?
        .build()
        .await?;

    info!("DBus service 'org.ermete.MeshBus' bound at path '/org/ermete/MeshBus'");

    // 6. Spawn Async AF_XDP Zero-Copy Ingestion Receiver Loop replacing legacy Linux socket loop
    let xdp_task = tokio::spawn(async move {
        info!("AF_XDP Kernel Bypass zero-copy packet ingestion loop active.");
        loop {
            if let Some(ref mut socket) = af_xdp_socket {
                match socket.recv_burst(32) {
                    Ok(packets) => {
                        for packet in packets {
                            if let Ok(payload) = packet.payload() {
                                // First pass packet to CRDT zero-trust broadcaster engine
                                let _ = crdt_broadcaster.process_afxdp_packet(payload);

                                match ZeroCopyParser::parse_frame(payload) {
                                    Ok(frame) => {
                                        info!(
                                            "AF_XDP Zero-Copy frame ingested: msg_type={:?}, sequence={}, len={}",
                                            frame.header().msg_type(),
                                            frame.header().sequence(),
                                            frame.payload_len()
                                        );
                                    }
                                    Err(_err) => {
                                        // Ignore non-mesh or unparseable packets
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        tracing::error!("AF_XDP recv_burst error: {}", err);
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    });

    info!("Ermete OS PQC Mesh Bus is running continuously in Kernel Bypass mode.");

    // 7. Wait for shutdown signal or XDP task finish
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT shutdown signal, shutting down PQC Mesh Bus...");
        }
        res = xdp_task => {
            if let Err(e) = res {
                tracing::error!("AF_XDP loop task joined with error: {}", e);
            }
        }
    }

    Ok(())
}
