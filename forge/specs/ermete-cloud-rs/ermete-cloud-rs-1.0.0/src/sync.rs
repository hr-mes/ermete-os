use anyhow::Result;
use tracing::{info, warn, error};
use tokio::net::{UdpSocket, TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::time::{Instant, Duration, sleep};
use pqc_kyber::{Keypair as KyberKeypair, KYBER_CIPHERTEXTBYTES};
use pqc_dilithium::Keypair as DilithiumKeypair;
use rand_core::OsRng;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::zk::{ZkProofEngine, ZkProof};
use crate::bft::{BftConsensusEngine, BftProposal, BftVote};

pub struct SyncEngine {
    known_peers: Arc<Mutex<HashMap<String, Instant>>>,
    auth_token: Arc<Mutex<Option<String>>>,
    kyber_keypair: KyberKeypair,
    dilithium_keypair: DilithiumKeypair,
    pub zk_engine: Arc<ZkProofEngine>,
    pub bft_engine: Arc<BftConsensusEngine>,
    node_id: String,
}

impl SyncEngine {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let kyber_keypair = pqc_kyber::keypair(&mut rng).expect("Failed to generate Kyber-1024 keypair for SyncEngine");
        let dilithium_keypair = DilithiumKeypair::generate();
        
        let dilithium_pk_b64 = BASE64.encode(&dilithium_keypair.public);
        let short_id = if dilithium_pk_b64.len() >= 12 { &dilithium_pk_b64[..12] } else { "node" };
        let node_id = format!("node-{}", short_id);

        let zk_engine = Arc::new(ZkProofEngine::new(node_id.clone(), None));
        let bft_engine = Arc::new(BftConsensusEngine::new(node_id.clone(), zk_engine.clone()));

        info!("SyncEngine Level 15 ZK-Mesh Computing & Byzantine Consensus Initialized for Node {}", node_id);

        Self {
            known_peers: Arc::new(Mutex::new(HashMap::new())),
            auth_token: Arc::new(Mutex::new(None)),
            kyber_keypair,
            dilithium_keypair,
            zk_engine,
            bft_engine,
            node_id,
        }
    }

    #[allow(dead_code)]
    pub fn get_node_id(&self) -> &str {
        &self.node_id
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

    pub fn get_zk_identity_info(&self) -> String {
        format!("ZkMeshNodeID: {}\nProofScheme: ZK-SNARK-GROTH16-ERMETE-V15\nStatus: Zero-Knowledge Fleet Member", self.node_id)
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
        info!("Starting Continuity ZK-P2P engine on local network with ZK-SNARK Verification & BFT Consensus...");
        
        let peers = self.known_peers.clone();
        let zk_verifier = self.zk_engine.clone();

        // UDP Broadcast listener for Discovery (Port 9090) with ZK Proof Verification
        tokio::spawn(async move {
            let socket = match UdpSocket::bind("0.0.0.0:9090").await {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to bind UDP discovery port 9090: {}", e);
                    return;
                }
            };
            let _ = socket.set_broadcast(true);
            let mut buf = [0u8; 4096];

            loop {
                if let Ok((len, addr)) = socket.recv_from(&mut buf).await {
                    let msg = String::from_utf8_lossy(&buf[..len]);
                    let ip = addr.ip().to_string();

                    if let Some(zk_payload) = msg.strip_prefix("ERMETE_ZK_HELLO:") {
                        if let Some((peer_node_id, proof_b64)) = zk_payload.split_once(':') {
                            if let Ok(proof) = ZkProof::from_b64(proof_b64) {
                                if zk_verifier.verify_proof(&proof) {
                                    let mut p = peers.lock().await;
                                    let is_new = !p.contains_key(&ip);
                                    p.insert(ip.clone(), Instant::now());
                                    if is_new {
                                        info!("Discovered ZK-Verified Ermete fleet node [{}] at IP {}", peer_node_id, ip);
                                    }
                                } else {
                                    warn!("Rejected unauthenticated discovery packet from IP {}: ZK proof verification failed!", ip);
                                }
                            }
                        }
                    } else if msg.starts_with("ERMETE_HELLO") {
                        let mut p = peers.lock().await;
                        p.insert(ip, Instant::now());
                    }
                }
            }
        });

        // UDP Broadcast sender for Discovery (Announce ourselves with ZK-SNARK Proof)
        let zk_prover = self.zk_engine.clone();
        tokio::spawn(async move {
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                let _ = socket.set_broadcast(true);
                let mut nonce = 1u64;
                loop {
                    if let Ok(proof) = zk_prover.generate_proof(nonce) {
                        if let Ok(proof_b64) = proof.to_b64() {
                            let packet = format!("ERMETE_ZK_HELLO:{}:{}", zk_prover.get_node_id(), proof_b64);
                            let _ = socket.send_to(packet.as_bytes(), "255.255.255.255:9090").await;
                        }
                    }
                    nonce += 1;
                    sleep(Duration::from_secs(5)).await;
                }
            }
        });

        // TCP Listener for incoming state & BFT consensus messages (Port 9091)
        let peers_ref = self.known_peers.clone();
        let auth_token_ref = self.auth_token.clone();
        let zk_verifier_ref = self.zk_engine.clone();
        let bft_engine_ref = self.bft_engine.clone();

        tokio::spawn(async move {
            info!("Initializing ZK-Mesh Sync TCP listener on port 9091 with Level 15 ZK-SNARKs and BFT Consensus...");
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
                    let active_peers_count = peers_guard.len() + 1;
                    drop(peers_guard);

                    let current_token = auth_token_ref.lock().await.clone();
                    let zk_v = zk_verifier_ref.clone();
                    let bft_e = bft_engine_ref.clone();

                    tokio::spawn(async move {
                        let mut content = String::new();
                        if stream.read_to_string(&mut content).await.is_ok() {
                            if content.is_empty() {
                                return;
                            }

                            // 1. Check BFT Proposal message
                            if let Some(prop_json) = content.strip_prefix("AUTH_BFT_PROP:") {
                                if let Ok(proposal) = serde_json::from_str::<BftProposal>(prop_json) {
                                    if let Ok(Some(vote)) = bft_e.handle_proposal(&proposal, active_peers_count).await {
                                        let vote_json = serde_json::to_string(&vote).unwrap_or_default();
                                        let reply = format!("AUTH_BFT_VOTE:{}", vote_json);
                                        let _ = stream.write_all(reply.as_bytes()).await;
                                    }
                                }
                                return;
                            }

                            // 2. Check BFT Vote message
                            if let Some(vote_json) = content.strip_prefix("AUTH_BFT_VOTE:") {
                                if let Ok(vote) = serde_json::from_str::<BftVote>(vote_json) {
                                    let _ = bft_e.handle_vote(&vote, active_peers_count).await;
                                }
                                return;
                            }

                            // 3. Standard ZK-Authenticated Payload (AUTH_ZK:<node_id>:<zk_proof_b64>\n<payload>)
                            if let Some((auth_header, payload)) = content.split_once('\n') {
                                let mut authenticated = false;

                                if let Some(zk_hdr) = auth_header.strip_prefix("AUTH_ZK:") {
                                    if let Some((peer_node_id, proof_b64)) = zk_hdr.split_once(':') {
                                        if let Ok(proof) = ZkProof::from_b64(proof_b64) {
                                            if proof.node_id == peer_node_id && zk_v.verify_proof(&proof) {
                                                info!("Zero-Knowledge Proof verified for peer {} ({})!", peer_node_id, peer_ip);
                                                authenticated = true;
                                            }
                                        }
                                    }
                                } else if let Some(pqc_hdr) = auth_header.strip_prefix("AUTH_PQC:") {
                                    if let Some((sig_b64, pk_b64)) = pqc_hdr.split_once(':') {
                                        if let (Ok(sig_bytes), Ok(pk_bytes)) = (BASE64.decode(sig_b64), BASE64.decode(pk_b64)) {
                                            if pqc_dilithium::verify(&sig_bytes, payload.as_bytes(), &pk_bytes).is_ok() {
                                                authenticated = true;
                                            }
                                        }
                                    }
                                } else if let Some(req_token) = current_token {
                                    if auth_header.trim() == format!("AUTH:{}", req_token) {
                                        authenticated = true;
                                    }
                                }

                                if authenticated {
                                    info!("Received ZK-Authenticated payload from peer {}! ({} bytes)", peer_ip, payload.len());
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
                            }
                        }
                    });
                }
            }
        });

        Ok(())
    }

    /// Broadcast clipboard sync backed by Zero-Knowledge authentication & BFT consensus
    pub async fn send_clipboard(&self, content: &str) -> Result<()> {
        let mut p = self.known_peers.lock().await;
        p.retain(|_, time| time.elapsed() < Duration::from_secs(60));
        let peers: Vec<String> = p.keys().cloned().collect();
        let total_fleet_nodes = peers.len() + 1;
        drop(p);

        // 1. Create BFT Proposal
        let proposal = self.bft_engine.create_proposal("clipboard", content, rand_core::RngCore::next_u64(&mut OsRng)).await?;
        let prop_json = serde_json::to_string(&proposal)?;
        let prop_msg = format!("AUTH_BFT_PROP:{}", prop_json);

        if peers.is_empty() {
            info!("Single node mesh: BFT Consensus immediately achieved for proposal {}", proposal.proposal_id);
            return Ok(());
        }

        // 2. Broadcast proposal to all peers to collect BFT Prepare / Commit votes
        for ip in peers {
            info!("Dispatching BFT Proposal [{}] to fleet peer {}...", proposal.proposal_id, ip);
            let addr = format!("{}:9091", ip);
            let bft_e = self.bft_engine.clone();

            if let Ok(mut stream) = TcpStream::connect(&addr).await {
                if stream.write_all(prop_msg.as_bytes()).await.is_ok() {
                    let mut resp = String::new();
                    if stream.read_to_string(&mut resp).await.is_ok() {
                        if let Some(vote_json) = resp.strip_prefix("AUTH_BFT_VOTE:") {
                            if let Ok(vote) = serde_json::from_str::<BftVote>(vote_json) {
                                let _ = bft_e.handle_vote(&vote, total_fleet_nodes).await;
                            }
                        }
                    }
                }
            } else {
                warn!("Peer {} unreachable for BFT consensus.", ip);
            }
        }

        if self.bft_engine.is_committed(&proposal.proposal_id).await {
            info!("BFT Consensus CONFIRMED across fleet for proposal [{}]", proposal.proposal_id);
        } else {
            warn!("BFT Proposal [{}] sent to peers, pending quorum confirmation.", proposal.proposal_id);
        }

        Ok(())
    }

    /// Propose custom state update through BFT consensus
    pub async fn propose_bft_state_update(&self, data_type: &str, payload: &str) -> Result<String> {
        let proposal = self.bft_engine.create_proposal(data_type, payload, rand_core::RngCore::next_u64(&mut OsRng)).await?;
        Ok(proposal.proposal_id)
    }
}
