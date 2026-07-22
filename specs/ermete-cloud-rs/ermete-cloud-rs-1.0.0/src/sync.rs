use anyhow::{Context, Result};
use tracing::{info, warn, error};
use tokio::net::{UdpSocket, TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashSet;
use std::net::SocketAddr;
use zbus::Connection;

pub struct SyncEngine {
    known_peers: Arc<Mutex<HashSet<String>>>,
}

impl SyncEngine {
    pub fn new() -> Self {
        Self {
            known_peers: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn start_discovery(&self) -> Result<()> {
        info!("Starting Continuity P2P engine on local network...");
        
        let peers = self.known_peers.clone();
        
        // UDP Broadcast listener for Discovery (Port 9090)
        tokio::spawn(async move {
            let socket = UdpSocket::bind("0.0.0.0:9090").await.expect("Failed to bind UDP");
            socket.set_broadcast(true).unwrap();
            let mut buf = [0; 1024];

            loop {
                if let Ok((len, addr)) = socket.recv_from(&mut buf).await {
                    let msg = String::from_utf8_lossy(&buf[..len]);
                    if msg.starts_with("ERMETE_HELLO") {
                        let ip = addr.ip().to_string();
                        let mut p = peers.lock().await;
                        if p.insert(ip.clone()) {
                            info!("Discovered new Ermete peer for Continuity: {}", ip);
                        }
                    }
                }
            }
        });

        // UDP Broadcast sender for Discovery (Announce ourselves)
        tokio::spawn(async move {
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                socket.set_broadcast(true).unwrap();
                loop {
                    let _ = socket.send_to(b"ERMETE_HELLO", "255.255.255.255:9090").await;
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        });

        // TCP Listener for incoming clipboard (Port 9091)
        tokio::spawn(async move {
            let listener = TcpListener::bind("0.0.0.0:9091").await.expect("Failed to bind TCP");
            loop {
                if let Ok((mut stream, _addr)) = listener.accept().await {
                    tokio::spawn(async move {
                        let mut content = String::new();
                        if stream.take(1024 * 1024).read_to_string(&mut content).await.is_ok() {
                            if content.is_empty() || content.contains('\0') {
                                warn!("Invalid clipboard payload");
                                return;
                            }
                            info!("Received Universal Clipboard from peer! ({} bytes)", content.len());
                            // Apply to local clipboard using wl-copy
                            if let Ok(mut child) = tokio::process::Command::new("wl-copy")
                                .stdin(std::process::Stdio::piped())
                                .spawn() 
                            {
                                if let Some(mut stdin) = child.stdin.take() {
                                    let _ = stdin.write_all(content.as_bytes()).await;
                                }
                            }
                        }
                    });
                }
            }
        });

        Ok(())
    }
    
    pub async fn send_clipboard(&self, content: &str) -> Result<()> {
        let peers = self.known_peers.lock().await.clone();
        
        if peers.is_empty() {
            warn!("No Ermete peers found on the local network for Continuity sync.");
            return Ok(());
        }

        for ip in peers {
            info!("Sending Universal Clipboard to peer {}...", ip);
            let addr = format!("{}:9091", ip);
            if let Ok(mut stream) = TcpStream::connect(&addr).await {
                if let Err(e) = stream.write_all(content.as_bytes()).await {
                    error!("Failed to send clipboard to {}: {}", ip, e);
                } else {
                    info!("Successfully pushed to {}", ip);
                }
            } else {
                warn!("Peer {} is unreachable via TCP.", ip);
            }
        }
        
        Ok(())
    }
}
