use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};
use tracing::{error, info, warn};

use crate::zk::{ZkProof, ZkProofEngine};

pub fn start_udp_discovery(
    peers: Arc<Mutex<HashMap<String, Instant>>>,
    zk_engine: Arc<ZkProofEngine>,
) {
    let peers_recv = peers.clone();
    let zk_verifier = zk_engine.clone();

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
                                let mut p = peers_recv.lock().await;
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
                    let mut p = peers_recv.lock().await;
                    p.insert(ip, Instant::now());
                }
            }
        }
    });

    // UDP Broadcast sender for Discovery (Announce ourselves with ZK-SNARK Proof)
    let zk_prover = zk_engine.clone();
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
}
