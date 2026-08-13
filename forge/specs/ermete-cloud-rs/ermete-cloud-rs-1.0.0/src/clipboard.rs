use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use tracing::{info, warn};
use rand_core::OsRng;

use crate::bft::{BftConsensusEngine, BftVote};

pub async fn broadcast_clipboard(
    known_peers: &Arc<Mutex<HashMap<String, Instant>>>,
    bft_engine: &Arc<BftConsensusEngine>,
    content: &str,
) -> Result<()> {
    let mut p = known_peers.lock().await;
    p.retain(|_, time| time.elapsed() < Duration::from_secs(60));
    let peers: Vec<String> = p.keys().cloned().collect();
    let total_fleet_nodes = peers.len() + 1;
    drop(p);

    // 1. Create BFT Proposal
    let proposal = bft_engine.create_proposal("clipboard", content, rand_core::RngCore::next_u64(&mut OsRng)).await?;
    let prop_json = serde_json::to_string(&proposal)?;
    let prop_msg = format!("AUTH_BFT_PROP:{}", prop_json);

    if peers.is_empty() {
        warn!("Single node mesh: Cannot achieve BFT quorum. Consensus requires multiple nodes.");
        return Err(anyhow::anyhow!("BFT Consensus failed: insufficient fleet peers."));
    }

    // 2. Broadcast proposal to all peers to collect BFT Prepare / Commit votes
    for ip in peers {
        info!("Dispatching BFT Proposal [{}] to fleet peer {}...", proposal.proposal_id, ip);
        let addr = format!("{}:9091", ip);
        let bft_e = bft_engine.clone();

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

    if bft_engine.is_committed(&proposal.proposal_id).await {
        info!("BFT Consensus CONFIRMED across fleet for proposal [{}]", proposal.proposal_id);
    } else {
        warn!("BFT Proposal [{}] sent to peers, pending quorum confirmation.", proposal.proposal_id);
    }

    Ok(())
}
