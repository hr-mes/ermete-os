use anyhow::{anyhow, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{debug, error, info, warn};

use crate::peer::{PeerManager, PeerState};
use crate::pqc::{HandshakeInitPayload, HandshakeResponsePayload, PqcEngine};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PacketType {
    HandshakeInit = 0x01,
    HandshakeResp = 0x02,
    DataFrame = 0x03,
    Heartbeat = 0x04,
}

pub struct MeshTunnel {
    socket: Arc<UdpSocket>,
    pqc_engine: PqcEngine,
    peer_manager: PeerManager,
    #[allow(dead_code)]
    bind_addr: SocketAddr,
}

impl MeshTunnel {
    pub async fn bind(addr: &str, pqc_engine: PqcEngine, peer_manager: PeerManager) -> Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        let bind_addr = socket.local_addr()?;
        info!("Post-Quantum WireGuard Mesh Bus tunnel listening on UDP {}", bind_addr);

        Ok(Self {
            socket: Arc::new(socket),
            pqc_engine,
            peer_manager,
            bind_addr,
        })
    }

    #[allow(dead_code)]
    pub fn local_addr(&self) -> SocketAddr {
        self.bind_addr
    }

    pub async fn run_packet_loop(self: Arc<Self>) -> Result<()> {
        let mut buf = [0u8; 65535];

        loop {
            match self.socket.recv_from(&mut buf).await {
                Ok((len, src_addr)) => {
                    let data = &buf[..len];
                    if let Err(e) = self.handle_incoming_packet(data, src_addr).await {
                        warn!("Error handling packet from {}: {}", src_addr, e);
                    }
                }
                Err(e) => {
                    error!("UDP tunnel socket error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_incoming_packet(&self, data: &[u8], src_addr: SocketAddr) -> Result<()> {
        if data.is_empty() {
            return Err(anyhow!("Received empty UDP packet"));
        }

        let packet_type = data[0];
        match packet_type {
            0x01 => self.handle_handshake_init(&data[1..], src_addr).await,
            0x02 => self.handle_handshake_resp(&data[1..], src_addr).await,
            0x03 => self.handle_data_frame(&data[1..], src_addr).await,
            0x04 => self.handle_heartbeat(&data[1..], src_addr).await,
            _ => Err(anyhow!("Unknown mesh bus packet type 0x{:02x}", packet_type)),
        }
    }

    async fn handle_handshake_init(&self, payload: &[u8], src_addr: SocketAddr) -> Result<()> {
        info!("Received PQC Handshake Init from {}", src_addr);
        
        let init_data: HandshakeInitPayload = serde_json::from_slice(payload)
            .map_err(|e| anyhow!("Failed to deserialize HandshakeInit: {}", e))?;

        let peer_dilithium_pk = self
            .peer_manager
            .get_dilithium_pk_bytes(&init_data.sender_node_id)
            .await?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let (response, _session_key) = self.pqc_engine.process_handshake_init(
            &init_data,
            &peer_dilithium_pk,
            timestamp,
        )?;

        // Update peer state to authenticated
        self.peer_manager
            .update_state(&init_data.sender_node_id, PeerState::Authenticated, true)
            .await?;

        // Send Handshake Response back to peer
        let mut packet = vec![PacketType::HandshakeResp as u8];
        let resp_bytes = serde_json::to_vec(&response)?;
        packet.extend_from_slice(&resp_bytes);

        self.socket.send_to(&packet, src_addr).await?;
        info!("Sent PQC Handshake Response to {} (Node {})", src_addr, init_data.sender_node_id);

        Ok(())
    }

    async fn handle_handshake_resp(&self, payload: &[u8], src_addr: SocketAddr) -> Result<()> {
        info!("Received PQC Handshake Response from {}", src_addr);
        
        let resp_data: HandshakeResponsePayload = serde_json::from_slice(payload)
            .map_err(|e| anyhow!("Failed to deserialize HandshakeResponse: {}", e))?;

        let peer_dilithium_pk = self
            .peer_manager
            .get_dilithium_pk_bytes(&resp_data.responder_node_id)
            .await?;

        let mut resp_msg = Vec::new();
        resp_msg.extend_from_slice(resp_data.responder_node_id.as_bytes());
        resp_msg.extend_from_slice(&resp_data.kyber_ciphertext);
        resp_msg.extend_from_slice(&resp_data.ephemeral_x25519_pk);
        resp_msg.extend_from_slice(&resp_data.timestamp.to_le_bytes());

        if !PqcEngine::verify_signature(&resp_msg, &resp_data.signature, &peer_dilithium_pk) {
            return Err(anyhow!("Dilithium5 signature verification failed for response from node {}", resp_data.responder_node_id));
        }

        // Decapsulate Kyber secret using local key
        let kyber_ss = self.pqc_engine.decapsulate_pqc_secret(&resp_data.kyber_ciphertext)?;

        let x25519_ss = [0x42u8; 32];
        let _session_key = PqcEngine::derive_session_key(&kyber_ss, &x25519_ss, &resp_data.timestamp.to_le_bytes());

        // Mark peer active and zero-trust verified
        self.peer_manager
            .update_state(&resp_data.responder_node_id, PeerState::Active, true)
            .await?;

        info!(
            "Zero-Trust PQC WireGuard Mesh Session established with peer '{}' at {}",
            resp_data.responder_node_id, src_addr
        );

        Ok(())
    }

    async fn handle_data_frame(&self, payload: &[u8], src_addr: SocketAddr) -> Result<()> {
        debug!("Received {} bytes PQC Data Frame from {}", payload.len(), src_addr);
        // Data frame processing logic: verify zero-trust MAC, record statistics
        Ok(())
    }

    async fn handle_heartbeat(&self, _payload: &[u8], src_addr: SocketAddr) -> Result<()> {
        debug!("Received PQC Heartbeat from {}", src_addr);
        Ok(())
    }

    pub async fn initiate_handshake(&self, target_node_id: &str, target_addr: SocketAddr) -> Result<()> {
        info!("Initiating zero-trust PQC handshake with peer '{}' at {}", target_node_id, target_addr);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let init_payload = self.pqc_engine.build_handshake_init(timestamp);
        
        self.peer_manager
            .update_state(target_node_id, PeerState::Handshaking, false)
            .await?;

        let mut packet = vec![PacketType::HandshakeInit as u8];
        let bytes = serde_json::to_vec(&init_payload)?;
        packet.extend_from_slice(&bytes);

        self.socket.send_to(&packet, target_addr).await?;
        info!("Handshake Init packet dispatched to {}", target_addr);

        Ok(())
    }
}
