use crate::types::{NpuCapabilities, SwarmBeacon, SwarmNode, SwarmNodeState};
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{error, info};

pub struct ZeroConfDiscovery {
    discovery_port: u16,
    ipc_port: u16,
    local_node_id: String,
    hostname: String,
    dilithium_pk_b64: String,
    kyber_pk_b64: String,
    x25519_pk_b64: String,
    npu_caps: NpuCapabilities,
}

impl ZeroConfDiscovery {
    pub fn new(
        discovery_port: u16,
        ipc_port: u16,
        local_node_id: String,
        hostname: String,
        dilithium_pk_b64: String,
        kyber_pk_b64: String,
        x25519_pk_b64: String,
        npu_caps: NpuCapabilities,
    ) -> Self {
        Self {
            discovery_port,
            ipc_port,
            local_node_id,
            hostname,
            dilithium_pk_b64,
            kyber_pk_b64,
            x25519_pk_b64,
            npu_caps,
        }
    }

    pub async fn start(
        self: Arc<Self>,
        swarm_manager: Arc<crate::swarm_manager::SwarmManager>,
    ) -> Result<()> {
        let bind_addr = format!("0.0.0.0:{}", self.discovery_port);
        let socket = Arc::new(UdpSocket::bind(&bind_addr).await?);
        socket.set_broadcast(true)?;

        info!(
            "ZeroConfDiscovery: Bound UDP socket on {} for P2P Ermete OS Swarm discovery",
            bind_addr
        );

        // 1. Spawn Outgoing Beacon Task (Every 3 seconds)
        let socket_tx = socket.clone();
        let self_tx = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
            loop {
                interval.tick().await;

                let beacon = SwarmBeacon {
                    node_id: self_tx.local_node_id.clone(),
                    hostname: self_tx.hostname.clone(),
                    endpoint_ip: "127.0.0.1".to_string(), // dynamically set or broadcast
                    discovery_port: self_tx.discovery_port,
                    ipc_port: self_tx.ipc_port,
                    dilithium_pk_b64: self_tx.dilithium_pk_b64.clone(),
                    kyber_pk_b64: self_tx.kyber_pk_b64.clone(),
                    x25519_pk_b64: self_tx.x25519_pk_b64.clone(),
                    npu_caps: self_tx.npu_caps.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                };

                if let Ok(bytes) = serde_json::to_vec(&beacon) {
                    let target_addr: SocketAddr = format!("255.255.255.255:{}", self_tx.discovery_port)
                        .parse()
                        .unwrap();
                    let _ = socket_tx.send_to(&bytes, target_addr).await;
                }
            }
        });

        // 2. Spawn Incoming Receiver Task
        let socket_rx = socket.clone();
        let self_rx = self.clone();
        tokio::spawn(async move {
            let mut buf = vec![0u8; 65536];
            loop {
                match socket_rx.recv_from(&mut buf).await {
                    Ok((n, src_addr)) => {
                        if let Ok(beacon) = serde_json::from_slice::<SwarmBeacon>(&buf[..n]) {
                            if beacon.node_id == self_rx.local_node_id {
                                continue; // Skip self broadcast
                            }

                            info!(
                                "Discovered Ermete OS Node '{}' ({}) via Zero-Conf at {} [NPU: {} {:.1} TOPS]",
                                beacon.node_id, beacon.hostname, src_addr.ip(), beacon.npu_caps.device_name, beacon.npu_caps.tops
                            );

                            let peer_node = SwarmNode {
                                node_id: beacon.node_id.clone(),
                                hostname: beacon.hostname.clone(),
                                endpoint_ip: src_addr.ip().to_string(),
                                ipc_port: beacon.ipc_port,
                                virtual_ip: None,
                                dilithium_pk_b64: beacon.dilithium_pk_b64.clone(),
                                kyber_pk_b64: beacon.kyber_pk_b64.clone(),
                                x25519_pk_b64: beacon.x25519_pk_b64.clone(),
                                npu_caps: beacon.npu_caps,
                                state: SwarmNodeState::Discovered,
                                assigned_layer_range: (0, 0),
                                last_seen_secs: beacon.timestamp,
                                pqc_verified: true,
                            };

                            swarm_manager.handle_discovered_peer(peer_node).await;
                        }
                    }
                    Err(e) => {
                        error!("ZeroConfDiscovery receive error: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}
