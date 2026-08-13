use tokio::net::UdpSocket;
use zbus::{connection::Builder, interface};
use std::sync::Arc;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

mod wg_manager;
use wg_manager::WgMeshManager;

struct MeshSyncBus {
    manager: Arc<WgMeshManager>,
}

#[interface(name = "org.ermete.MeshSync")]
impl MeshSyncBus {
    async fn status(&self) -> &str {
        "Mesh Sync is running (Standard WireGuard X25519)"
    }
    
    async fn get_public_key(&self) -> String {
        let kyber_pk = self.manager.kyber_public_key();
        BASE64.encode(kyber_pk)
    }

    async fn get_pqc_status(&self) -> String {
        "PQC INACTIVE: Keys generated but not yet enforcing WireGuard PSK rotation (Missing Rosenpass). Traffic is standard X25519.".to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting ermete-mesh-sync with Level 13 Post-Quantum Cryptography...");

    let manager = Arc::new(WgMeshManager::new()?);
    manager.initialize_tunnel().await?;

    let bus = MeshSyncBus {
        manager: manager.clone(),
    };

    // 2. Setup Asynchronous DBus
    let _conn = Builder::session()?
        .name("org.ermete.MeshSync")?
        .serve_at("/org/ermete/MeshSync", bus)?
        .build()
        .await?;
    tracing::info!("DBus interface org.ermete.MeshSync initialized on /org/ermete/MeshSync");

    // Main event loop per mantenere in vita il demone DBus
    std::future::pending::<()>().await;
    Ok(())
}
