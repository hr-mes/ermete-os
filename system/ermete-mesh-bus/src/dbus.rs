use crate::peer::PeerManager;
use crate::pqc::PqcEngine;
use crate::tunnel::MeshTunnel;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use zbus::interface;
use zbus::zvariant::{OwnedValue, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PolkitSubject {
    pub kind: String,
    pub details: HashMap<String, OwnedValue>,
}

impl PolkitSubject {
    pub fn system_bus_name(name: impl Into<String>) -> Self {
        let mut details = HashMap::new();
        let val: Value = Value::from(name.into());
        if let Ok(owned) = val.try_into() {
            details.insert("name".to_string(), owned);
        }
        Self {
            kind: "system-bus-name".to_string(),
            details,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PolkitAuthorizationResult {
    pub is_authorized: bool,
    pub is_challenge: bool,
    pub details: HashMap<String, String>,
}

#[zbus::proxy(
    interface = "org.freedesktop.PolicyKit1.Authority",
    default_service = "org.freedesktop.PolicyKit1",
    default_path = "/org/freedesktop/PolicyKit1/Authority"
)]
pub trait PolicyKitAuthority {
    fn check_authorization(
        &self,
        subject: &PolkitSubject,
        action_id: &str,
        details: &HashMap<&str, &str>,
        flags: u32,
        cancellation_id: &str,
    ) -> zbus::Result<PolkitAuthorizationResult>;
}

pub async fn check_polkit_auth_zbus(
    conn: &zbus::Connection,
    sender: &str,
    action_id: &str,
    allow_user_interaction: bool,
) -> Result<bool, zbus::Error> {
    if let Ok(creds) = conn.peer_credentials().await {
        if creds.uid() == Some(0) {
            return Ok(true);
        }
    }

    let proxy = PolicyKitAuthorityProxy::new(conn).await?;
    let subject = PolkitSubject::system_bus_name(sender);
    let details = HashMap::<&str, &str>::new();
    let flags = if allow_user_interaction { 1u32 } else { 0u32 };

    let result = proxy
        .check_authorization(&subject, action_id, &details, flags, "")
        .await?;

    Ok(result.is_authorized)
}

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
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
        node_id: String,
        endpoint: String,
        dilithium_pk_b64: String,
        kyber_pk_b64: String,
        x25519_pk_b64: String,
    ) -> zbus::fdo::Result<String> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.meshbus.manage", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;
        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for add_peer".into()));
        }

        let ep = if endpoint.is_empty() { None } else { Some(endpoint) };
        match self
            .peer_manager
            .add_peer(node_id, ep, dilithium_pk_b64, kyber_pk_b64, x25519_pk_b64)
            .await
        {
            Ok(peer) => Ok(serde_json::to_string(&peer).unwrap_or_default()),
            Err(e) => Err(zbus::fdo::Error::Failed(format!("Failed to add peer: {}", e))),
        }
    }

    async fn remove_peer(
        &self,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
        node_id: String,
    ) -> zbus::fdo::Result<String> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.meshbus.manage", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;
        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for remove_peer".into()));
        }

        match self.peer_manager.remove_peer(&node_id).await {
            Ok(_) => Ok(format!("Peer '{}' successfully removed", node_id)),
            Err(e) => Err(zbus::fdo::Error::Failed(format!("Failed to remove peer: {}", e))),
        }
    }

    async fn initiate_handshake(
        &self,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
        node_id: String,
        endpoint: String,
    ) -> zbus::fdo::Result<String> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.meshbus.manage", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;
        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for initiate_handshake".into()));
        }

        let tunnel = match &self.tunnel {
            Some(t) => t,
            None => return Err(zbus::fdo::Error::Failed("Mesh tunnel socket not initialized".into())),
        };

        let addr: SocketAddr = match endpoint.parse() {
            Ok(a) => a,
            Err(e) => return Err(zbus::fdo::Error::InvalidArgs(format!("Invalid endpoint address format: {}", e))),
        };

        match tunnel.initiate_handshake(&node_id, addr).await {
            Ok(_) => Ok(format!("Handshake initiated with peer '{}' at {}", node_id, addr)),
            Err(e) => Err(zbus::fdo::Error::Failed(format!("Handshake failed: {}", e))),
        }
    }

    async fn get_node_identity(&self) -> String {
        let identity = self.pqc_engine.get_node_identity();
        serde_json::to_string_pretty(&identity).unwrap_or_default()
    }

    async fn rotate_keys(
        &mut self,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<String> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.meshbus.manage", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;
        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for rotate_keys".into()));
        }

        self.pqc_engine
            .rotate_keys()
            .map_err(|e| zbus::fdo::Error::Failed(format!("PQC key rotation failed: {}", e)))?;

        Ok(format!(
            "PQC Keys rotated for node '{}'. New ML-KEM-1024 and Dilithium5 keypairs active.",
            self.pqc_engine.node_id()
        ))
    }
}
