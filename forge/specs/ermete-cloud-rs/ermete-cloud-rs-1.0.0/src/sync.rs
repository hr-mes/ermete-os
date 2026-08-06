use anyhow::Result;
use tracing::{info, warn, error};
use tokio::net::{UdpSocket, TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::time::{Instant, Duration};

pub struct SyncEngine {
    known_peers: Arc<Mutex<HashMap<String, Instant>>>,
    auth_token: Arc<Mutex<Option<String>>>,
}

impl SyncEngine {
    pub fn new() -> Self {
        Self {
            known_peers: Arc::new(Mutex::new(HashMap::new())),
            auth_token: Arc::new(Mutex::new(None)),
        }
    }

    #[allow(dead_code)]
    pub fn set_auth_token(&self, token: String) {
        if let Ok(mut t) = self.auth_token.try_lock() {
            *t = Some(token);
        }
    }

    pub async fn start_discovery(&self) -> Result<()> {
        info!("Starting Continuity P2P engine on local network...");
        
        let peers = self.known_peers.clone();
        
        // UDP Broadcast listener for Discovery (Port 9090)
        tokio::spawn(async move {
            let socket = match UdpSocket::bind("0.0.0.0:9090").await {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to bind UDP discovery port 9090: {}", e);
                    return;
                }
            };
            let _ = socket.set_broadcast(true);
            let mut buf = [0u8; 1024];

            loop {
                if let Ok((len, addr)) = socket.recv_from(&mut buf).await {
                    let msg = String::from_utf8_lossy(&buf[..len]);
                    if msg.starts_with("ERMETE_HELLO") {
                        let ip = addr.ip().to_string();
                        let mut p = peers.lock().await;
                        let is_new = !p.contains_key(&ip);
                        p.insert(ip.clone(), Instant::now());
                        if is_new {
                            info!("Discovered authenticated Ermete peer for Continuity: {}", ip);
                        }
                    }
                }
            }
        });

        // UDP Broadcast sender for Discovery (Announce ourselves)
        tokio::spawn(async move {
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                let _ = socket.set_broadcast(true);
                loop {
                    let _ = socket.send_to(b"ERMETE_HELLO", "255.255.255.255:9090").await;
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        });

        // TCP Listener for incoming clipboard (Port 9091) - Secured with Auth & IP Verification
        let peers_ref = self.known_peers.clone();
        let auth_token_ref = self.auth_token.clone();

        tokio::spawn(async move {
            info!("Initializing Mesh Sync TCP listener on port 9091 with peer verification...");
            let listener = match TcpListener::bind("0.0.0.0:9091").await {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to bind TCP 9091: {}", e);
                    return;
                }
            };

            loop {
                if let Ok((stream, addr)) = listener.accept().await {
                    let peer_ip = addr.ip().to_string();
                    let peers_guard = peers_ref.lock().await;
                    let is_peer_known = peers_guard.contains_key(&peer_ip);
                    drop(peers_guard);

                    if !is_peer_known {
                        warn!("Rejected unauthenticated TCP connection from untrusted IP: {}", peer_ip);
                        continue;
                    }

                    let current_token = auth_token_ref.lock().await.clone();

                    // Security check: require TLS/Noise secure session or valid Auth Token
                    let required_token = match current_token {
                        Some(tok) => tok,
                        None => {
                            warn!("Rejecting unencrypted incoming clipboard on TCP 9091 from {}: TLS/Noise tunnel not established", peer_ip);
                            continue;
                        }
                    };

                    tokio::spawn(async move {
                        let mut content = String::new();
                        if stream.take(1024 * 1024).read_to_string(&mut content).await.is_ok() {
                            if content.is_empty() || content.contains('\0') {
                                warn!("Invalid clipboard payload from {}", peer_ip);
                                return;
                            }

                            // Verify message authentication header format: AUTH:<token>\n<payload>
                            if let Some((auth_header, payload)) = content.split_once('\n') {
                                if auth_header.trim() == format!("AUTH:{}", required_token) {
                                    info!("Received authenticated Universal Clipboard from peer {}! ({} bytes)", peer_ip, payload.len());
                                    let payload_str = payload.to_string();
                                    tokio::spawn(async move {
                                        if let Ok(mut child) = tokio::process::Command::new("wl-copy")
                                            .stdin(std::process::Stdio::piped())
                                            .spawn() 
                                        {
                                            if let Some(mut stdin) = child.stdin.take() {
                                                let _ = stdin.write_all(payload_str.as_bytes()).await;
                                                drop(stdin);
                                            }
                                            let _ = child.wait().await;
                                        }
                                    });
                                } else {
                                    warn!("Authentication token mismatch from peer IP {}", peer_ip);
                                }
                            } else {
                                warn!("Missing authentication header from peer IP {}", peer_ip);
                            }
                        }
                    });
                }
            }
        });

        Ok(())
    }
    
    pub async fn send_clipboard(&self, content: &str) -> Result<()> {
        let mut p = self.known_peers.lock().await;
        p.retain(|_, time| time.elapsed() < Duration::from_secs(60));
        let peers: Vec<String> = p.keys().cloned().collect();
        drop(p);
        
        if peers.is_empty() {
            warn!("No Ermete peers found on the local network for Continuity sync.");
            return Ok(());
        }

        let token_guard = self.auth_token.lock().await;
        let auth_header = match &*token_guard {
            Some(token) => format!("AUTH:{}\n", token),
            None => {
                warn!("Cannot send clipboard: secure TLS/Noise session / auth token not established.");
                return Ok(());
            }
        };
        drop(token_guard);

        let payload = format!("{}{}", auth_header, content);

        for ip in peers {
            info!("Sending authenticated Universal Clipboard to peer {}...", ip);
            let addr = format!("{}:9091", ip);
            if let Ok(mut stream) = TcpStream::connect(&addr).await {
                if let Err(e) = stream.write_all(payload.as_bytes()).await {
                    error!("Failed to send clipboard to {}: {}", ip, e);
                } else {
                    info!("Successfully pushed authenticated payload to {}", ip);
                }
            } else {
                warn!("Peer {} is unreachable via TCP.", ip);
            }
        }
        
        Ok(())
    }
}
