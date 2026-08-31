use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{error, info, warn};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use subtle::ConstantTimeEq;

use crate::bft::{BftConsensusEngine, BftProposal, BftVote};
use crate::zk::{ZkProof, ZkProofEngine};

pub fn start_tcp_listener(
    peers: Arc<Mutex<HashMap<String, Instant>>>,
    auth_token: Arc<Mutex<Option<String>>>,
    zk_engine: Arc<ZkProofEngine>,
    bft_engine: Arc<BftConsensusEngine>,
) {
    let peers_ref = peers.clone();
    let auth_token_ref = auth_token.clone();
    let zk_verifier_ref = zk_engine.clone();
    let bft_engine_ref = bft_engine.clone();

    tokio::spawn(async move {
        info!("Initializing Mesh Sync TCP listener on port 9091 with Dilithium Auth and BFT Consensus...");
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
                    if tokio::io::AsyncReadExt::read_to_string(&mut tokio::io::AsyncReadExt::take(&mut stream, 1024 * 1024 * 10), &mut content).await.is_ok() {
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
                                let expected = format!("AUTH:{}", req_token);
                                if bool::from(auth_header.trim().as_bytes().ct_eq(expected.as_bytes())) {
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
}

