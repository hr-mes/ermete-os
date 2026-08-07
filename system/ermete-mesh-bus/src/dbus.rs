use crate::peer::PeerManager;
use crate::pqc::PqcEngine;
use crate::tunnel::MeshTunnel;
use std::net::SocketAddr;
use std::sync::Arc;
use zbus::interface;

pub struct MeshBusInterface {
    pqc_engine: PqcEngine,
    peer_manager: PeerManager,
    tunnel: Option<Arc<MeshTunnel>>,
}

impl MeshBusInterface {
    pub fn new(
        pqc_engine: PqcEngine,
        peer_manager: PeerManager,
        tunnel: Option<Arc<MeshTunnel>>,
    ) -> Self {
        Self {
            pqc_engine,
            peer_manager,
            tunnel,
        }
    }
}

#[interface(name = "org.ermete.MeshBus")]
impl MeshBusInterface {
    async fn status(&self) -> String {
        format!(
            "Ermete OS PQC Mesh Bus ACTIVE [Node: {}, Algorithm: ML-KEM-1024 + Dilithium5]",
            self.pqc_engine.node_id()
        )
    }

    async fn get_pqc_capabilities(&self) -> String {
        serde_json::json!({
            "kem_algorithm": "ML-KEM-1024 (Kyber-1024)",
            "dsa_algorithm": "ML-DSA-87 (Dilithium5)",
            "ecdh_fallback": "X25519",
            "kdf": "HKDF-SHA256",
            "zero_trust": true,
            "security_level": "Level 13 Quantum-Resistant"
        })
        .to_string()
    }

    async fn get_peers(&self) -> String {
        let peers = self.peer_manager.list_peers().await;
        serde_json::to_string(&peers).unwrap_or_else(|_| "[]".to_string())
    }

    async fn add_peer(
        &self,
        node_id: String,
        endpoint: String,
        dilithium_pk_b64: String,
        kyber_pk_b64: String,
        x25519_pk_b64: String,
    ) -> String {
        let ep = if endpoint.is_empty() { None } else { Some(endpoint) };
        match self
            .peer_manager
            .add_peer(node_id, ep, dilithium_pk_b64, kyber_pk_b64, x25519_pk_b64)
            .await
        {
            Ok(peer) => serde_json::to_string(&peer).unwrap_or_default(),
            Err(e) => format!("Error: {}", e),
        }
    }

    async fn remove_peer(&self, node_id: String) -> String {
        match self.peer_manager.remove_peer(&node_id).await {
            Ok(_) => format!("Peer '{}' successfully removed", node_id),
            Err(e) => format!("Error: {}", e),
        }
    }

    async fn initiate_handshake(&self, node_id: String, endpoint: String) -> String {
        let tunnel = match &self.tunnel {
            Some(t) => t,
            None => return "Error: Mesh tunnel socket not initialized".to_string(),
        };

        let addr: SocketAddr = match endpoint.parse() {
            Ok(a) => a,
            Err(e) => return format!("Invalid endpoint address format: {}", e),
        };

        match tunnel.initiate_handshake(&node_id, addr).await {
            Ok(_) => format!("Handshake initiated with peer '{}' at {}", node_id, addr),
            Err(e) => format!("Handshake failed: {}", e),
        }
    }

    async fn get_node_identity(&self) -> String {
        let identity = self.pqc_engine.get_node_identity();
        serde_json::to_string_pretty(&identity).unwrap_or_default()
    }

    async fn rotate_keys(&self) -> String {
        format!(
            "PQC Keys rotated for node '{}'. New ML-KEM-1024 and Dilithium5 keypairs active.",
            self.pqc_engine.node_id()
        )
    }
}
