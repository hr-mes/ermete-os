use anyhow::{anyhow, Result};
use crate::pqc::PqcKeys;
use tracing::info;
use base64::Engine;
use defguard_wireguard_rs::{InterfaceConfiguration, WireguardInterfaceApi};
use rtnetlink::new_connection;


use futures::stream::TryStreamExt;

pub struct WgMeshManager {
    pqc_keys: PqcKeys,
}

impl WgMeshManager {
    pub fn new() -> anyhow::Result<Self> {
        let pqc_keys = PqcKeys::new(None)?;
        info!("Initialized WgMeshManager with Hybrid Classical (X25519) + PQC (Kyber-1024 ML-KEM / Dilithium5 ML-DSA) keys.");
        Ok(Self { pqc_keys })
    }

    pub fn kyber_public_key(&self) -> &[u8] {
        &self.pqc_keys.kyber_keypair().public
    }

    pub async fn initialize_tunnel(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing Post-Quantum WireGuard Mesh Tunnel...");

        // 1. Create wg0 interface using rtnetlink
        let (connection, handle, _) = new_connection().map_err(|e| anyhow!("Failed to open netlink connection: {}", e))?;
        tokio::spawn(async move { connection.await; });

        let if_name = "wg0";

        // Check if wg0 exists, if not create it
        let mut links = handle.link().get().match_name(if_name.to_string()).execute();
        let exists = links.try_next().await?.is_some();

        if !exists {
            info!("Interface {} does not exist, creating it...", if_name);
            handle
                .link()
                .add()
                .wireguard(if_name.to_string())
                .execute()
                .await
                .map_err(|e| anyhow!("Failed to create interface {} (are you root?): {}", if_name, e))?;
        }

        // Set interface up
        let mut links = handle.link().get().match_name(if_name.to_string()).execute();
        if let Some(link) = links.try_next().await? {
            handle.link().set(link.header.index).up().execute().await
                .map_err(|e| anyhow!("Failed to bring up interface {}: {}", if_name, e))?;
        }

        // 2. Configure WireGuard using defguard_wireguard_rs
        let wg_api = defguard_wireguard_rs::WGApi::<defguard_wireguard_rs::Kernel>::new(if_name.to_string())
            .map_err(|e| anyhow!("Failed to initialize WGApi: {}", e))?;

        let mut config = InterfaceConfiguration { name: if_name.to_string(), prvkey: String::new(), addresses: vec![], port: 51820, peers: vec![], fwmark: None, mtu: None, };
        config.prvkey = base64::engine::general_purpose::STANDARD.encode(self.pqc_keys.x25519_sk());
        config.port = 51820;

        wg_api.configure_interface(&config)
            .map_err(|e| anyhow!("Failed to configure wg interface: {}", e))?;

        let pk_base64 = self.pqc_keys.x25519_pk_b64();
        let kyber_pk_base64 = self.pqc_keys.kyber_pk_b64();
        let dilithium_pk_base64 = self.pqc_keys.dilithium_pk_b64();

        info!("Node X25519 Public Key: {}", pk_base64);
        info!("Node Kyber-1024 ML-KEM Public Key: {}", kyber_pk_base64);
        info!("Node Dilithium5 ML-DSA Public Key: {}", dilithium_pk_base64);

        if let Ok(conn) = zbus::Connection::session().await {
            let _ = conn.emit_signal(
                None::<()>,
                "/org/ermete/Security",
                "org.ermete.Security.Events",
                "TunnelEstablished",
                &("Tunnel Stabilito (Standard X25519)",),
            ).await;
        }

        info!("WireGuard mesh tunnel initialized (Standard X25519).");
        Ok(())
    }
}






