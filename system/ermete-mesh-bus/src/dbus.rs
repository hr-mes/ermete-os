//! `org.ermete.MeshBus` D-Bus interface: peer management (polkit-gated) and status queries.

use crate::peer::PeerManager;

use crate::tunnel::MeshTunnel;
use std::net::SocketAddr;
use std::sync::Arc;
use ermete_bus_api::polkit::check_polkit_auth_zbus;
use zbus::interface;

/// Implements the `org.ermete.MeshBus` D-Bus interface backing peer management and status queries.
pub struct MeshBusInterface {
    node_id: String,
    peer_manager: PeerManager,
    tunnel: Option<Arc<MeshTunnel>>,
}

impl MeshBusInterface {
    /// Constructs the interface handler over a shared [`PeerManager`] and optional
    /// [`MeshTunnel`] (`None` when the UDP tunnel failed to bind at startup).
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
