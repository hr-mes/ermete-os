use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{info, warn};
use crate::zk::{ZkProofEngine, ZkProof};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BftVoteType {
    Prepare,
    Commit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BftProposalState {
    Proposed,
    Prepared,
    Committed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BftProposal {
    pub proposal_id: String,
    pub proposer_id: String,
    pub data_type: String,
    pub payload: String,
    pub epoch: u64,
    pub sequence: u64,
    pub zk_proof: ZkProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BftVote {
    pub proposal_id: String,
    pub voter_id: String,
    pub vote_type: BftVoteType,
    pub approved: bool,
    pub zk_proof: ZkProof,
}

pub struct BftConsensusEngine {
    node_id: String,
    zk_engine: Arc<ZkProofEngine>,
    proposals: Arc<Mutex<HashMap<String, BftProposalRecord>>>,
    committed_history: Arc<Mutex<Vec<String>>>,
    sequence_counter: Arc<Mutex<u64>>,
}

struct BftProposalRecord {
    proposal: BftProposal,
    state: BftProposalState,
    prepare_votes: HashSet<String>,
    commit_votes: HashSet<String>,
    #[allow(dead_code)]
    created_at: Instant,
}

impl BftConsensusEngine {
    pub fn new(node_id: String, zk_engine: Arc<ZkProofEngine>) -> Self {
        info!("Initialized Byzantine Fault Tolerance (BFT) Consensus Engine for node {}", node_id);
        Self {
            node_id,
            zk_engine,
            proposals: Arc::new(Mutex::new(HashMap::new())),
            committed_history: Arc::new(Mutex::new(Vec::new())),
            sequence_counter: Arc::new(Mutex::new(1)),
        }
    }

    /// Spawns background worker sweeping old proposal records (older than 300s) to prevent memory leak
    pub fn spawn_proposal_pruner(self: &Arc<Self>) {
        let engine = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            let ttl = tokio::time::Duration::from_secs(300);

            loop {
                interval.tick().await;
                let mut proposals = engine.proposals.lock().await;
                let now = Instant::now();
                proposals.retain(|_, record| now.duration_since(record.created_at) < ttl);
            }
        });
    }

    /// Calculate Byzantine Fault Tolerance Quorum Threshold (2f + 1)
    /// N = total active nodes, f = max faulty nodes
    pub fn calculate_quorum(total_nodes: usize) -> usize {
        if total_nodes <= 1 {
            1
        } else {
            // Standard BFT supermajority threshold: ceil(2/3 * N)
            ((2 * total_nodes) / 3) + 1
        }
    }

    /// Create a new proposal for BFT Consensus validation across the fleet
    pub async fn create_proposal(&self, data_type: &str, payload: &str, nonce: u64) -> Result<BftProposal> {
        let mut seq_guard = self.sequence_counter.lock().await;
        let seq = *seq_guard;
        *seq_guard += 1;

        let zk_proof = self.zk_engine.generate_proof(nonce)?;
        let proposal_id = format!("BFT_PROP_{}_{}", self.node_id, seq);

        let proposal = BftProposal {
            proposal_id: proposal_id.clone(),
            proposer_id: self.node_id.clone(),
            data_type: data_type.to_string(),
            payload: payload.to_string(),
            epoch: 1,
            sequence: seq,
            zk_proof: zk_proof.clone(),
        };

        let mut record = BftProposalRecord {
            proposal: proposal.clone(),
            state: BftProposalState::Proposed,
            prepare_votes: HashSet::new(),
            commit_votes: HashSet::new(),
            created_at: Instant::now(),
        };
        // Proposer self-votes Prepare & Commit
        record.prepare_votes.insert(self.node_id.clone());
        record.commit_votes.insert(self.node_id.clone());

        self.proposals.lock().await.insert(proposal_id, record);

        info!("Created BFT Consensus Proposal [{}] for data_type '{}'", proposal.proposal_id, data_type);
        Ok(proposal)
    }

    /// Receive and validate incoming proposal from a fleet peer
    pub async fn handle_proposal(&self, proposal: &BftProposal, total_fleet_peers: usize) -> Result<Option<BftVote>> {
        // 1. Validate ZK proof of proposer
        if !self.zk_engine.verify_proof(&proposal.zk_proof) {
            warn!("Rejected BFT proposal {}: Invalid ZK Proof from proposer {}", proposal.proposal_id, proposal.proposer_id);
            return Ok(None);
        }

        let mut proposals = self.proposals.lock().await;
        if proposals.contains_key(&proposal.proposal_id) {
            return Ok(None);
        }

        info!("Received valid BFT Proposal [{}] from peer node {}", proposal.proposal_id, proposal.proposer_id);

        let mut record = BftProposalRecord {
            proposal: proposal.clone(),
            state: BftProposalState::Proposed,
            prepare_votes: HashSet::new(),
            commit_votes: HashSet::new(),
            created_at: Instant::now(),
        };

        // Automatic PREPARE vote if proposal is valid
        record.prepare_votes.insert(proposal.proposer_id.clone());
        record.prepare_votes.insert(self.node_id.clone());
        
        let prepare_vote = BftVote {
            proposal_id: proposal.proposal_id.clone(),
            voter_id: self.node_id.clone(),
            vote_type: BftVoteType::Prepare,
            approved: true,
            zk_proof: self.zk_engine.generate_proof(proposal.sequence)?,
        };

        let quorum = Self::calculate_quorum(total_fleet_peers);
        if record.prepare_votes.len() >= quorum {
            record.state = BftProposalState::Prepared;
            info!("BFT Proposal [{}] reached PREPARED state (votes: {}/{})", proposal.proposal_id, record.prepare_votes.len(), quorum);
        }

        proposals.insert(proposal.proposal_id.clone(), record);
        Ok(Some(prepare_vote))
    }

    /// Receive and process vote from a peer
    pub async fn handle_vote(&self, vote: &BftVote, total_fleet_peers: usize) -> Result<Option<BftVote>> {
        if !self.zk_engine.verify_proof(&vote.zk_proof) {
            warn!("Rejected BFT vote from {}: Invalid ZK Proof", vote.voter_id);
            return Ok(None);
        }

        let quorum = Self::calculate_quorum(total_fleet_peers);
        let mut proposals = self.proposals.lock().await;
        let record = match proposals.get_mut(&vote.proposal_id) {
            Some(r) => r,
            None => {
                warn!("Received vote for unknown BFT proposal: {}", vote.proposal_id);
                return Ok(None);
            }
        };

        match vote.vote_type {
            BftVoteType::Prepare => {
                if vote.approved {
                    record.prepare_votes.insert(vote.voter_id.clone());
                }
                if record.state == BftProposalState::Proposed && record.prepare_votes.len() >= quorum {
                    record.state = BftProposalState::Prepared;
                    info!("BFT Proposal [{}] transition to PREPARED quorum achieved ({}/{})", vote.proposal_id, record.prepare_votes.len(), quorum);

                    // Broadcast COMMIT vote
                    let commit_vote = BftVote {
                        proposal_id: vote.proposal_id.clone(),
                        voter_id: self.node_id.clone(),
                        vote_type: BftVoteType::Commit,
                        approved: true,
                        zk_proof: self.zk_engine.generate_proof(record.proposal.sequence)?,
                    };
                    record.commit_votes.insert(self.node_id.clone());
                    return Ok(Some(commit_vote));
                }
            }
            BftVoteType::Commit => {
                if vote.approved {
                    record.commit_votes.insert(vote.voter_id.clone());
                }
                if record.state != BftProposalState::Committed && record.commit_votes.len() >= quorum {
                    record.state = BftProposalState::Committed;
                    info!("BFT Consensus ACHIEVED for Proposal [{}] (Commit quorum {}/{})!", vote.proposal_id, record.commit_votes.len(), quorum);

                    self.committed_history.lock().await.push(vote.proposal_id.clone());
                }
            }
        }

        Ok(None)
    }

    /// Check if a proposal has achieved BFT consensus commitment
    pub async fn is_committed(&self, proposal_id: &str) -> bool {
        let proposals = self.proposals.lock().await;
        if let Some(r) = proposals.get(proposal_id) {
            r.state == BftProposalState::Committed
        } else {
            false
        }
    }

    pub async fn get_status(&self) -> String {
        let proposals = self.proposals.lock().await;
        let committed = self.committed_history.lock().await;
        format!(
            "BFT Consensus Engine Node: {}\nTotal Proposals Tracked: {}\nTotal Committed: {}\nState: Active BFT Quorum",
            self.node_id,
            proposals.len(),
            committed.len()
        )
    }
}
