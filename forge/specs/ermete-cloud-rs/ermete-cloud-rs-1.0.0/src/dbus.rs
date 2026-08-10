use zbus::interface;
use tracing::info;
use crate::sync::SyncEngine;
use crate::zk::ZkProof;
use std::sync::Arc;

pub struct CloudIface {
    pub engine: Arc<SyncEngine>,
}

#[interface(name = "os.ermete.Cloud")]
impl CloudIface {
    /// Syncs local clipboard to trusted peers with Dilithium authentication & BFT consensus
    async fn push_clipboard(&self, content: String) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to push clipboard with Dilithium proof and BFT consensus.");
        
        match self.engine.send_clipboard(&content).await {
            Ok(_) => Ok("Clipboard pushed to fleet with Dilithium proof & BFT consensus validation.".into()),
            Err(e) => Ok(format!("Error: {}", e)),
        }
    }

    /// Exposes PQC Kyber-1024 public key
    async fn get_pqc_kyber_public_key(&self) -> std::result::Result<String, zbus::fdo::Error> {
        Ok(self.engine.get_kyber_public_key_b64())
    }

    /// Exposes PQC Dilithium5 public key
    async fn get_pqc_dilithium_public_key(&self) -> std::result::Result<String, zbus::fdo::Error> {
        Ok(self.engine.get_dilithium_public_key_b64())
    }

    /// Exposes node ZK Mesh identity
    async fn get_zk_identity(&self) -> std::result::Result<String, zbus::fdo::Error> {
        Ok(self.engine.get_zk_identity_info())
    }

    /// Exposes current BFT Consensus engine status
    async fn get_bft_status(&self) -> std::result::Result<String, zbus::fdo::Error> {
        Ok(self.engine.bft_engine.get_status().await)
    }

    /// Propose state update via Byzantine Fault Tolerance
    async fn propose_bft_update(&self, data_type: String, payload: String) -> std::result::Result<String, zbus::fdo::Error> {
        match self.engine.propose_bft_state_update(&data_type, &payload).await {
            Ok(prop_id) => Ok(format!("BFT Proposal created: {}", prop_id)),
            Err(e) => Ok(format!("Failed to create BFT proposal: {}", e)),
        }
    }

    /// Verify an arbitrary ZK Proof string
    async fn verify_zk_proof(&self, proof_b64: String) -> std::result::Result<bool, zbus::fdo::Error> {
        match ZkProof::from_b64(&proof_b64) {
            Ok(proof) => Ok(self.engine.zk_engine.verify_proof(&proof)),
            Err(_) => Ok(false),
        }
    }
}
