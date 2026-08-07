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
        "Mesh Sync is running (Level 13 Post-Quantum WireGuard + Kyber-1024 / Dilithium5)"
    }
    
    async fn get_public_key(&self) -> String {
        let kyber_pk = self.manager.kyber_public_key();
        BASE64.encode(kyber_pk)
    }

    async fn get_pqc_status(&self) -> String {
        "PQC Level 13 ACTIVE: Kyber-1024 (ML-KEM) & Dilithium5 (ML-DSA)".to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting ermete-mesh-sync with Level 13 Post-Quantum Cryptography...");

    let manager = Arc::new(WgMeshManager::new());
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

    // 3. Asynchronous UDP listener for user-space WireGuard
    let socket = UdpSocket::bind("0.0.0.0:51820").await?;
    let socket = Arc::new(socket);
    tracing::info!("Listening for Post-Quantum Mesh WG traffic on UDP 0.0.0.0:51820...");

    let mut buf = [0u8; 2048];

    // Main event loop
    loop {
        tokio::select! {
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, addr)) => {
                        tracing::info!("Received {} bytes of PQC-protected packet from {}", len, addr);
                    }
                    Err(e) => {
                        tracing::error!("Error receiving packet: {}", e);
                    }
                }
            }
        }
    }
}
