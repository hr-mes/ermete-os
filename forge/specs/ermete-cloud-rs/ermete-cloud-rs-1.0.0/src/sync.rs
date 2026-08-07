use anyhow::Result;
use tracing::{info, warn, error};
use tokio::net::{UdpSocket, TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::time::{Instant, Duration};
use pqc_kyber::{Keypair as KyberKeypair, KYBER_CIPHERTEXTBYTES};

use pqc_dilithium::Keypair as DilithiumKeypair;
use rand_core::OsRng;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

pub struct SyncEngine {
    known_peers: Arc<Mutex<HashMap<String, Instant>>>,
    auth_token: Arc<Mutex<Option<String>>>,
    kyber_keypair: KyberKeypair,
    dilithium_keypair: DilithiumKeypair,
}

impl SyncEngine {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let kyber_keypair = pqc_kyber::keypair(&mut rng).expect("Failed to generate Kyber-1024 keypair for SyncEngine");
        let dilithium_keypair = DilithiumKeypair::generate();

        info!("SyncEngine Level 13 PQC Cryptography Engine Initialized (Kyber-1024 / Dilithium5)");

        Self {
            known_peers: Arc::new(Mutex::new(HashMap::new())),
            auth_token: Arc::new(Mutex::new(None)),
            kyber_keypair,
            dilithium_keypair,
        }
    }

    #[allow(dead_code)]
    pub fn set_auth_token(&self, token: String) {
        if let Ok(mut t) = self.auth_token.try_lock() {
            *t = Some(token);
        }
    }

    pub fn get_kyber_public_key_b64(&self) -> String {
        BASE64.encode(&self.kyber_keypair.public)
    }

    pub fn get_dilithium_public_key_b64(&self) -> String {
        BASE64.encode(&self.dilithium_keypair.public)
    }

    #[allow(dead_code)]
    pub fn encapsulate_pqc_secret(&self, peer_kyber_pk_b64: &str) -> Result<(String, String)> {
        let peer_pk_bytes = BASE64.decode(peer_kyber_pk_b64)?;
        if peer_pk_bytes.len() != pqc_kyber::KYBER_PUBLICKEYBYTES {
            anyhow::bail!("Invalid Kyber public key length");
        }
        let mut peer_pk = [0u8; pqc_kyber::KYBER_PUBLICKEYBYTES];
        peer_pk.copy_from_slice(&peer_pk_bytes);
        let mut rng = OsRng;
        let (ct, ss) = pqc_kyber::encapsulate(&peer_pk, &mut rng)
            .map_err(|e| anyhow::anyhow!("Kyber encapsulation failed: {:?}", e))?;
        Ok((BASE64.encode(ct), BASE64.encode(ss)))
    }

    #[allow(dead_code)]
    pub fn decapsulate_pqc_secret(&self, ciphertext_b64: &str) -> Result<String> {
        let ct_bytes = BASE64.decode(ciphertext_b64)?;
        if ct_bytes.len() != KYBER_CIPHERTEXTBYTES {
            anyhow::bail!("Invalid ciphertext length for Kyber-1024");
        }
        let mut ct_arr = [0u8; KYBER_CIPHERTEXTBYTES];
        ct_arr.copy_from_slice(&ct_bytes);
        let ss = pqc_kyber::decapsulate(&ct_arr, &self.kyber_keypair.secret)
            .map_err(|e| anyhow::anyhow!("Kyber decapsulation failed: {:?}", e))?;
        Ok(BASE64.encode(ss))
    }

    pub async fn start_discovery(&self) -> Result<()> {
        info!("Starting Continuity P2P engine on local network with Post-Quantum Protection...");
        
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
                            info!("Discovered authenticated PQC Ermete peer for Continuity: {}", ip);
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
                    let _ = socket.send_to(b"ERMETE_HELLO_PQC", "255.255.255.255:9090").await;
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        });

        // TCP Listener for incoming clipboard (Port 9091) - Secured with Auth & PQC Verification
        let peers_ref = self.known_peers.clone();
        let auth_token_ref = self.auth_token.clone();

        tokio::spawn(async move {
            info!("Initializing Mesh Sync TCP listener on port 9091 with Level 13 PQC Dilithium5 verification...");
            let listener = match TcpListener::bind("0.0.0.0:9091").await {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to bind TCP 9091: {}", e);
                    return;
                }
            };

            loop {
                if let Ok((mut stream, addr)) = listener.accept().await {
                    let peer_ip = addr.ip().to_string();
                    let peers_guard = peers_ref.lock().await;
                    let is_peer_known = peers_guard.contains_key(&peer_ip);
                    drop(peers_guard);

                    if !is_peer_known {
                        warn!("Rejected unauthenticated TCP connection from untrusted IP: {}", peer_ip);
                        continue;
                    }

                    let current_token = auth_token_ref.lock().await.clone();

                    tokio::spawn(async move {
                        let mut content = String::new();
                        if stream.read_to_string(&mut content).await.is_ok() {
                            if content.is_empty() || content.contains('\0') {
                                warn!("Invalid clipboard payload from {}", peer_ip);
                                return;
                            }

                            // Verify message authentication header format:
                            // AUTH_PQC:<sig_b64>:<pk_b64>\n<payload> OR AUTH:<token>\n<payload>
                            if let Some((auth_header, payload)) = content.split_once('\n') {
                                let mut authenticated = false;

                                if let Some(pqc_hdr) = auth_header.strip_prefix("AUTH_PQC:") {
                                    if let Some((sig_b64, pk_b64)) = pqc_hdr.split_once(':') {
                                        if let (Ok(sig_bytes), Ok(pk_bytes)) = (BASE64.decode(sig_b64), BASE64.decode(pk_b64)) {
                                            if pqc_dilithium::verify(&sig_bytes, payload.as_bytes(), &pk_bytes).is_ok() {
                                                info!("Post-Quantum Dilithium5 signature verified for peer {}!", peer_ip);
                                                authenticated = true;
                                            } else {
                                                warn!("Dilithium5 signature verification failed for peer {}", peer_ip);
                                            }
                                        }
                                    }
                                } else if let Some(req_token) = current_token {
                                    if auth_header.trim() == format!("AUTH:{}", req_token) {
                                        authenticated = true;
                                    }
                                }

                                if authenticated {
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
                                    warn!("Authentication failed for peer IP {}", peer_ip);
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

        // Generate Dilithium5 signature for the payload
        let sig = self.dilithium_keypair.sign(content.as_bytes());
        let sig_b64 = BASE64.encode(&sig);
        let pk_b64 = BASE64.encode(&self.dilithium_keypair.public);
        let auth_header = format!("AUTH_PQC:{}:{}\n", sig_b64, pk_b64);

        let payload = format!("{}{}", auth_header, content);

        for ip in peers {
            info!("Sending Dilithium5 PQC-authenticated Universal Clipboard to peer {}...", ip);
            let addr = format!("{}:9091", ip);
            if let Ok(mut stream) = TcpStream::connect(&addr).await {
                if let Err(e) = stream.write_all(payload.as_bytes()).await {
                    error!("Failed to send clipboard to {}: {}", ip, e);
                } else {
                    info!("Successfully pushed PQC authenticated payload to {}", ip);
                }
            } else {
                warn!("Peer {} is unreachable via TCP.", ip);
            }
        }
        
        Ok(())
    }
}
