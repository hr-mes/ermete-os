use anyhow::Result;
use tracing::info;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::time::Instant;
use pqc_kyber::Keypair as KyberKeypair;
use pqc_dilithium::Keypair as DilithiumKeypair;
use rand_core::OsRng;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::zk::ZkProofEngine;
use crate::bft::BftConsensusEngine;
use crate::discovery;
use crate::listener;
use crate::clipboard;

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
    pub fn new() -> Result<Self> {
        let mut rng = OsRng;
        let kyber_keypair = pqc_kyber::keypair(&mut rng)
            .map_err(|e| anyhow::anyhow!("Failed to generate Kyber-1024 keypair for SyncEngine: {:?}", e))?;
        let dilithium_keypair = DilithiumKeypair::generate();
        
        let dilithium_pk_b64 = BASE64.encode(dilithium_keypair.public);
        let short_id = if dilithium_pk_b64.len() >= 12 { &dilithium_pk_b64[..12] } else { "node" };
        let node_id = format!("node-{}", short_id);

        let zk_engine = Arc::new(ZkProofEngine::new(node_id.clone(), None));
        let bft_engine = Arc::new(BftConsensusEngine::new(node_id.clone(), zk_engine.clone()));

        info!("SyncEngine Level 15 ZK-Mesh Computing & Byzantine Consensus Initialized for Node {}", node_id);

        Ok(Self {
            known_peers: Arc::new(Mutex::new(HashMap::new())),
            auth_token: Arc::new(Mutex::new(None)),
            kyber_keypair,
            dilithium_keypair,
            zk_engine,
            bft_engine,
            node_id,
        })
    }

    pub fn get_kyber_public_key_b64(&self) -> String {
        BASE64.encode(self.kyber_keypair.public)
    }

    pub fn get_dilithium_public_key_b64(&self) -> String {
        BASE64.encode(self.dilithium_keypair.public)
    }

    pub fn get_zk_identity_info(&self) -> String {
        format!("Node: {} (Dilithium Auth enabled)", self.node_id)
    }

    pub async fn start_discovery(&self) -> Result<()> {
        info!("Starting Continuity P2P engine on local network with Dilithium Verification & BFT Consensus...");
        
        self.bft_engine.spawn_proposal_pruner();
        discovery::start_udp_discovery(self.known_peers.clone(), self.zk_engine.clone());
        listener::start_tcp_listener(
            self.known_peers.clone(),
            self.auth_token.clone(),
            self.zk_engine.clone(),
            self.bft_engine.clone(),
        );

        Ok(())
    }

    /// Broadcast clipboard sync backed by Zero-Knowledge authentication & BFT consensus
    pub async fn send_clipboard(&self, content: &str) -> Result<()> {
        clipboard::broadcast_clipboard(&self.known_peers, &self.bft_engine, content).await
    }

    /// Propose custom state update through BFT consensus
    pub async fn propose_bft_state_update(&self, data_type: &str, payload: &str) -> Result<String> {
        let proposal = self.bft_engine.create_proposal(data_type, payload, rand_core::RngCore::next_u64(&mut OsRng)).await?;
        Ok(proposal.proposal_id)
    }
}
