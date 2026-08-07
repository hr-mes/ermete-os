use anyhow::{anyhow, Result};
use tracing::{info, warn};
use zbus::Connection;

#[derive(Clone)]
pub struct PqcMeshClient {
    dbus_service_name: String,
    dbus_path: String,
}

impl PqcMeshClient {
    pub fn new() -> Self {
        Self {
            dbus_service_name: "org.ermete.MeshBus".to_string(),
            dbus_path: "/org/ermete/MeshBus".to_string(),
        }
    }

    pub async fn check_status(&self) -> Result<String> {
        let connection = Connection::session().await?;
        let reply: String = connection
            .call_method(
                Some(self.dbus_service_name.as_str()),
                self.dbus_path.as_str(),
                Some("org.ermete.MeshBus"),
                "status",
                &(),
            )
            .await?
            .body()
            .deserialize()?;
        Ok(reply)
    }

    #[allow(dead_code)]
    pub async fn get_local_identity(&self) -> Result<serde_json::Value> {
        let connection = Connection::session().await?;
        let reply: String = connection
            .call_method(
                Some(self.dbus_service_name.as_str()),
                self.dbus_path.as_str(),
                Some("org.ermete.MeshBus"),
                "get_node_identity",
                &(),
            )
            .await?
            .body()
            .deserialize()?;
        let value: serde_json::Value = serde_json::from_str(&reply)?;
        Ok(value)
    }

    pub async fn register_and_handshake_peer(
        &self,
        node_id: &str,
        endpoint: &str,
        dilithium_pk_b64: &str,
        kyber_pk_b64: &str,
        x25519_pk_b64: &str,
    ) -> Result<()> {
        let connection = match Connection::session().await {
            Ok(conn) => conn,
            Err(e) => {
                warn!("Unable to connect to Session DBus for MeshBus: {}", e);
                return Err(anyhow!("DBus session connection failed: {}", e));
            }
        };

        info!(
            "PqcMeshClient: Registering peer '{}' at '{}' with org.ermete.MeshBus",
            node_id, endpoint
        );

        // 1. Call add_peer
        let add_res: String = connection
            .call_method(
                Some(self.dbus_service_name.as_str()),
                self.dbus_path.as_str(),
                Some("org.ermete.MeshBus"),
                "add_peer",
                &(
                    node_id.to_string(),
                    endpoint.to_string(),
                    dilithium_pk_b64.to_string(),
                    kyber_pk_b64.to_string(),
                    x25519_pk_b64.to_string(),
                ),
            )
            .await?
            .body()
            .deserialize()?;

        info!("MeshBus add_peer response: {}", add_res);

        // 2. Call initiate_handshake
        let hs_res: String = connection
            .call_method(
                Some(self.dbus_service_name.as_str()),
                self.dbus_path.as_str(),
                Some("org.ermete.MeshBus"),
                "initiate_handshake",
                &(node_id.to_string(), endpoint.to_string()),
            )
            .await?
            .body()
            .deserialize()?;

        info!("MeshBus initiate_handshake response: {}", hs_res);

        Ok(())
    }
}
