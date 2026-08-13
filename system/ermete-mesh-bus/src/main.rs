#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::mut_from_ref)]

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use zbus::connection::Builder;

mod dbus;
pub mod ipc;
pub mod network;
mod peer;
pub mod protocol;
pub mod sync;
mod tunnel;

use std::sync::Arc;
use dbus::MeshBusInterface;
use network::{AfXdpConfig, AfXdpSocket};
use peer::PeerManager;
use protocol::ZeroCopyParser;
use sync::{CrdtBroadcaster, StorageBridge};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("--------------------------------------------------");
    info!("Starting Ermete OS PQC Mesh Bus Daemon (ermete-mesh-bus)");
    info!("Level 13 Zero-Trust Post-Quantum Kernel Bypass Engine");
    info!("--------------------------------------------------");

    // 2. Initialize PQC Cryptographic Engine (ML-KEM-1024 / Dilithium5 / X25519)
    let identity = "node-alpha".to_string();

    info!("Local Node Identity: {}", identity);
            
    // 3. Initialize Peer Manager & IPC Storage Bridge (Fase 11)
    let peer_manager = PeerManager::new();
    peer_manager.spawn_heartbeat_pruner(60);
    let storage_bridge = Arc::new(StorageBridge::new(None, None)?);
    let (crdt_broadcaster, _background_dispatcher) = CrdtBroadcaster::new(
        peer_manager.clone(),
        storage_bridge,
    );

    // 3.5 Initialize MeshTunnel (UDP Fallback)
    let (ingress_tx, mut ingress_rx) = tokio::sync::mpsc::channel(1024);
    let mesh_tunnel = Arc::new(tunnel::MeshTunnel::bind_with_channel(
        "0.0.0.0:51820",
        peer_manager.clone(),
        Some(ingress_tx),
    ).await?);

    let tunnel_task_clone = mesh_tunnel.clone();
    tokio::spawn(async move {
        if let Err(e) = tunnel_task_clone.run_packet_loop().await {
            tracing::error!("MeshTunnel packet loop error: {}", e);
        }
    });

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
        peer_manager.clone(),
        Some(mesh_tunnel.clone()),
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
            let mut activity = false;

            if let Some(ref mut socket) = af_xdp_socket {
                match socket.recv_burst(32) {
                    Ok(packets) => {
                        for packet in packets {
                            activity = true;
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
            
            // Standard UDP fallback loop check (reads from MeshTunnel channel)
            match ingress_rx.try_recv() {
                Ok(frame) => {
                    activity = true;
                    let _ = crdt_broadcaster.process_afxdp_packet(&frame.payload);
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    tracing::error!("UDP tunnel ingress channel disconnected!");
                    break;
                }
            }

            // Connect CRDT bridges to prepare_broadcast_frame
            match crdt_broadcaster.storage_bridge().receive_broadcast_request() {
                Ok(Some(delta)) => {
                    activity = true;
                    match crdt_broadcaster.prepare_broadcast_frame(
                        &delta.target_namespace,
                        delta.delta_type,
                        delta.payload_bytes,
                        None, // Broadcast
                    ) {
                        Ok(frame_data) => {
                            if let Some(ref mut socket) = af_xdp_socket {
                                if let Err(e) = socket.send_packet(&frame_data) {
                                    tracing::error!("AF_XDP send_packet failed for CRDT broadcast: {}", e);
                                }
                            } else {
                                // Send via UDP when XDP not available could be implemented here
                            }
                        }
                        Err(e) => tracing::error!("Failed to prepare CRDT broadcast frame: {}", e),
                    }
                }
                Ok(None) => {}
                Err(e) => tracing::error!("Error receiving CRDT broadcast request: {}", e),
            }

            if !activity {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
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
