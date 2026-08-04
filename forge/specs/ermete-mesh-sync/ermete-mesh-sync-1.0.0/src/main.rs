use tokio::net::UdpSocket;
use zbus::{connection::Builder, interface};
use x25519_dalek::{EphemeralSecret, PublicKey};
use rand_core::OsRng;
use std::sync::Arc;

struct MeshSyncBus;

#[interface(name = "org.ermete.MeshSync")]
impl MeshSyncBus {
    async fn status(&self) -> &str {
        "Mesh Sync is running (Async WireGuard)"
    }
    
    async fn get_public_key(&self) -> String {
        // In a real app this would be shared state
        "placeholder-key".to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting ermete-mesh-sync in async mode...");

    // 1. Generate X25519 Keys for WireGuard
    let secret_key = EphemeralSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&secret_key);
    println!("Generated WireGuard public key.");

    // 2. Setup Asynchronous DBus
    let _conn = Builder::session()?
        .name("org.ermete.MeshSync")?
        .serve_at("/org/ermete/MeshSync", MeshSyncBus)?
        .build()
        .await?;
    println!("DBus interface org.ermete.MeshSync initialized on /org/ermete/MeshSync");

    // 3. Asynchronous UDP listener for user-space WireGuard (boringtun setup placeholder)
    let socket = UdpSocket::bind("0.0.0.0:51820").await?;
    let socket = Arc::new(socket);
    println!("Listening for Mesh WG traffic on UDP 0.0.0.0:51820...");

    let mut buf = [0u8; 2048];

    // Main event loop (Non-blocking)
    loop {
        tokio::select! {
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, addr)) => {
                        println!("Received {} bytes from {}", len, addr);
                        // TODO: Route packet through boringtun's DeviceHandle
                        // ...
                    }
                    Err(e) => {
                        eprintln!("Error receiving packet: {}", e);
                    }
                }
            }
            // Add other async event handlers here (e.g., dbus signal propagation)
        }
    }
}
