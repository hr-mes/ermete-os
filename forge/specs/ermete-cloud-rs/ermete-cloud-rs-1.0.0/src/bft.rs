//! Byzantine Fault Tolerant (2-phase Prepare/Commit) consensus engine for validating clipboard and
//! other state-update proposals across the fleet, gated on this crate's ZK-proof authentication.
//! Note this is the *intended* path — the clipboard write itself (`wl-copy`) is never actually
//! invoked from anywhere in this file; the only code path that writes to the clipboard is the
//! separate, directly-TCP-reachable branch in `listener.rs` (see `AUDIT_REPORT.md` SEC-01/SEC-07).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{info, warn};
use crate::zk::{ZkProofEngine, ZkProof};

/// Phase of the two-phase BFT vote: `Prepare` (initial quorum) then `Commit` (final quorum).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BftVoteType {
    Prepare,
    Commit,
}

/// Lifecycle state of a BFT proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BftProposalState {
    /// Just created/received, not yet at Prepare quorum.
    Proposed,
    /// Prepare-vote quorum reached; Commit votes are being collected.
    Prepared,
    /// Commit-vote quorum reached; consensus achieved.
    Committed,
    /// Not currently set by any code path in this file.
    Rejected,
}

/// A proposed state update (e.g. a clipboard push) submitted for BFT consensus, authenticated by
/// the proposer's ZK proof.
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

/// A Prepare or Commit vote on a [`BftProposal`], itself authenticated by the voter's ZK proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BftVote {
    pub proposal_id: String,
    pub voter_id: String,
    pub vote_type: BftVoteType,
    pub approved: bool,
    pub zk_proof: ZkProof,
}

/// Tracks in-flight and committed BFT proposals for this node.
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
    /// Creates an empty consensus engine for `node_id`, backed by `zk_engine` for authenticating
    /// proposals and votes.
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

    /// Create a new proposal for BFT Consensus validation across the fleet. The proposer
    /// automatically self-votes both Prepare and Commit.
    ///
    /// # Errors
    /// Returns an error if generating the ZK proof fails (e.g. system clock before UNIX epoch).
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

    /// Receive and validate incoming proposal from a fleet peer. Returns `Ok(None)` if the ZK
    /// proof fails verification or the proposal was already known; otherwise records it and
    /// returns a Prepare vote to send back.
    ///
    /// # Errors
    /// Returns an error if generating this node's own Prepare-vote ZK proof fails.
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

    /// Receive and process vote from a peer. Returns `Ok(None)` if the ZK proof fails
    /// verification or the referenced proposal is unknown; on reaching Prepare quorum, returns a
    /// Commit vote to broadcast; on reaching Commit quorum, records the proposal as committed and
    /// returns `Ok(None)` (the commit itself is only observable via [`Self::is_committed`]).
    ///
    /// # Errors
    /// Returns an error if generating this node's own Commit-vote ZK proof fails.
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

    /// Returns a human-readable summary of tracked/committed proposal counts.
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
