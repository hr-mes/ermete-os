use anyhow::{anyhow, Result};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerState {
    Discovered,
    Handshaking,
    Authenticated,
    Active,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub node_id: String,
    pub endpoint: Option<String>,
    pub virtual_ip: String,
    pub dilithium_pk_b64: String,
    pub kyber_pk_b64: String,
    pub x25519_pk_b64: String,
    pub state: PeerState,
    pub last_handshake: u64,
    pub packets_rx: u64,
    pub packets_tx: u64,
    pub latency_ms: u32,
    pub zero_trust_verified: bool,
}

#[derive(Clone)]
pub struct PeerManager {
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    ip_counter: Arc<RwLock<u8>>,
}

impl PeerManager {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            ip_counter: Arc::new(RwLock::new(2)), // Starts at 10.99.0.2
        }
    }

    pub async fn add_peer(
        &self,
        node_id: String,
        endpoint: Option<String>,
        dilithium_pk_b64: String,
        kyber_pk_b64: String,
        x25519_pk_b64: String,
    ) -> Result<Peer> {
        let mut peers = self.peers.write().await;
        if peers.contains_key(&node_id) {
            return Err(anyhow!("Peer '{}' already registered in mesh bus", node_id));
        }

        let mut counter = self.ip_counter.write().await;
        let vip = format!("10.99.0.{}", *counter);
        *counter += 1;

        let peer = Peer {
            node_id: node_id.clone(),
            endpoint,
            virtual_ip: vip,
            dilithium_pk_b64,
            kyber_pk_b64,
            x25519_pk_b64,
            state: PeerState::Discovered,
            last_handshake: 0,
            packets_rx: 0,
            packets_tx: 0,
            latency_ms: 0,
            zero_trust_verified: false,
        };

        peers.insert(node_id.clone(), peer.clone());
        info!("Added zero-trust mesh peer '{}' with virtual IP {}", node_id, peer.virtual_ip);

        Ok(peer)
    }

    pub async fn remove_peer(&self, node_id: &str) -> Result<()> {
        let mut peers = self.peers.write().await;
        if peers.remove(node_id).is_some() {
            info!("Removed peer '{}' from mesh bus", node_id);
            Ok(())
        } else {
            Err(anyhow!("Peer '{}' not found", node_id))
        }
    }

    pub async fn update_state(&self, node_id: &str, state: PeerState, verified: bool) -> Result<()> {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(node_id) {
            peer.state = state;
            peer.zero_trust_verified = verified;
            if verified {
                peer.last_handshake = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
            }
            Ok(())
        } else {
            Err(anyhow!("Peer '{}' not found", node_id))
        }
    }

    #[allow(dead_code)]
    pub async fn update_stats(&self, node_id: &str, rx_bytes: u64, tx_bytes: u64) {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(node_id) {
            peer.packets_rx += rx_bytes;
            peer.packets_tx += tx_bytes;
        }
    }

    pub async fn get_peer(&self, node_id: &str) -> Option<Peer> {
        let peers = self.peers.read().await;
        peers.get(node_id).cloned()
    }

    pub async fn list_peers(&self) -> Vec<Peer> {
        let peers = self.peers.read().await;
        peers.values().cloned().collect()
    }

    pub async fn get_dilithium_pk_bytes(&self, node_id: &str) -> Result<Vec<u8>> {
        let peer = self.get_peer(node_id).await.ok_or_else(|| anyhow!("Peer not found"))?;
        BASE64
            .decode(&peer.dilithium_pk_b64)
            .map_err(|e| anyhow!("Invalid Dilithium base64 for peer {}: {}", node_id, e))
    }

    #[allow(dead_code)]
    pub async fn get_kyber_pk_bytes(&self, node_id: &str) -> Result<Vec<u8>> {
        let peer = self.get_peer(node_id).await.ok_or_else(|| anyhow!("Peer not found"))?;
        BASE64
            .decode(&peer.kyber_pk_b64)
            .map_err(|e| anyhow!("Invalid Kyber base64 for peer {}: {}", node_id, e))
    }

    /// Spawns an asynchronous background worker that periodically purges inactive peers
    /// exceeding the specified heartbeat/handshake timeout in seconds.
    pub fn spawn_heartbeat_pruner(&self, timeout_secs: u64) {
        let peers_ref = self.peers.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
            loop {
                interval.tick().await;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let mut peers = peers_ref.write().await;
                let before_count = peers.len();
                peers.retain(|node_id, peer| {
                    if peer.last_handshake == 0 {
                        return true;
                    }
                    let active = (now - peer.last_handshake) < timeout_secs;
                    if !active {
                        info!(
                            "Mesh Bus: Pruned dead peer '{}' (no heartbeat/handshake for >{}s)",
                            node_id, timeout_secs
                        );
                    }
                    active
                });
                let pruned = before_count - peers.len();
                if pruned > 0 {
                    info!("PeerManager sweep completed: pruned {} dead peer(s)", pruned);
                }
            }
        });
    }
}
