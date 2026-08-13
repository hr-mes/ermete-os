use crate::peer::PeerManager;

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
    pub fn unix_process(pid: u32) -> Self {
        let mut details = std::collections::HashMap::new();
        if let Ok(owned) = zbus::zvariant::Value::from(pid).try_into() {
            details.insert("pid".to_string(), owned);
        }
        Self {
            kind: "unix-process".to_string(),
            details,
        }
    }

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
    if let Ok(creds) = conn.peer_creds().await {
        if creds.unix_user_id() == Some(0) {
            return Ok(true);
        }
    }

    let proxy = PolicyKitAuthorityProxy::new(conn).await?;
    let subject = if let Ok(creds) = conn.peer_creds().await {
        if let Some(pid) = creds.process_id() {
            PolkitSubject::unix_process(pid)
        } else {
            PolkitSubject::system_bus_name(sender)
        }
    } else {
        PolkitSubject::system_bus_name(sender)
    };
    let details = HashMap::<&str, &str>::new();
    let flags = if allow_user_interaction { 1u32 } else { 0u32 };

    let result = proxy
        .check_authorization(&subject, action_id, &details, flags, "")
        .await?;

    Ok(result.is_authorized)
}

pub struct MeshBusInterface {
    node_id: String,
    peer_manager: PeerManager,
    tunnel: Option<Arc<MeshTunnel>>,
}

impl MeshBusInterface {
    pub fn new(
        node_id: String,
        peer_manager: PeerManager,
        tunnel: Option<Arc<MeshTunnel>>,
    ) -> Self {
        Self {
            node_id,
            peer_manager,
            tunnel,
        }
    }
}

#[interface(name = "org.ermete.MeshBus")]
impl MeshBusInterface {
    async fn status(&self) -> String {
        format!(
            "Ermete OS Mesh Bus ACTIVE [Node: {}, WireGuard/X25519]",
            self.node_id
        )
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
            .add_peer(node_id, ep, x25519_pk_b64)
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

}
