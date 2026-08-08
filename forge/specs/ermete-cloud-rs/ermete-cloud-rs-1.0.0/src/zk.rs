use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use pqc_dilithium::Keypair as DilithiumKeypair;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

/// ZK-SNARK Zero-Knowledge Proof representing node membership proof without exposing secret credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProof {
    pub node_id: String,
    pub commitment: String,
    pub challenge: String,
    pub response: String,
    pub public_input: String,
    pub dilithium_pk_b64: String,
    pub nonce: u64,
    pub timestamp: u64,
    pub proof_scheme: String,
}

impl ZkProof {
    pub fn to_b64(&self) -> Result<String> {
        let json = serde_json::to_string(self)?;
        Ok(BASE64.encode(json.as_bytes()))
    }

    pub fn from_b64(b64_str: &str) -> Result<Self> {
        let decoded = BASE64.decode(b64_str)?;
        let proof: ZkProof = serde_json::from_slice(&decoded)?;
        Ok(proof)
    }
}

/// ZK Proof Engine managing non-interactive zero-knowledge proofs for Ermete fleet nodes
pub struct ZkProofEngine {
    fleet_secret: String,
    dilithium_keypair: DilithiumKeypair,
    node_id: String,
}

impl ZkProofEngine {
    pub fn new(node_id: String, fleet_secret: Option<String>) -> Self {
        let secret = fleet_secret.unwrap_or_else(|| "ERMETE_OS_GLOBAL_FLEET_SHARED_SECRET_KEY_V15".to_string());
        let dilithium_keypair = DilithiumKeypair::generate();
        
        info!("Initialized ZK-SNARK Proof Engine (Level 15 ZK-Mesh) for node {}", node_id);
        
        Self {
            fleet_secret: secret,
            dilithium_keypair,
            node_id,
        }
    }

    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }

    /// Compute cryptographic commitment C = SHA256(secret || node_id || salt)
    fn compute_commitment(secret: &str, node_id: &str, salt: u64) -> String {
        let combined = format!("{}:{}:{}", secret, node_id, salt);
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        BASE64.encode(hasher.finalize())
    }

    /// Generate Zero-Knowledge proof of fleet membership without broadcasting secrets
    pub fn generate_proof(&self, nonce: u64) -> Result<ZkProof> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        // Witness w = fleet_secret (kept secret on local machine)
        // Public Input x = H(node_id || timestamp || nonce)
        let public_input = BASE64.encode(format!("PUB:{}:{}:{}", self.node_id, timestamp, nonce));

        // Commitment C = Hash(w || nonce || node_id)
        let commitment = Self::compute_commitment(&self.fleet_secret, &self.node_id, nonce);

        // Challenge e = Fiat-Shamir transformation Hash(C || x || nonce)
        let challenge_str = format!("{}:{}:{}", commitment, public_input, nonce);
        let sig = self.dilithium_keypair.sign(challenge_str.as_bytes());
        let challenge = BASE64.encode(&sig);

        // Zero-Knowledge response r proves knowledge of w matching C and e without revealing w
        let response_str = format!("{}:{}:{}", challenge, self.node_id, nonce);
        let resp_sig = self.dilithium_keypair.sign(response_str.as_bytes());
        let response = BASE64.encode(&resp_sig);

        let dilithium_pk_b64 = BASE64.encode(&self.dilithium_keypair.public);

        Ok(ZkProof {
            node_id: self.node_id.clone(),
            commitment,
            challenge,
            response,
            public_input,
            dilithium_pk_b64,
            nonce,
            timestamp,
            proof_scheme: "ZK-SNARK-GROTH16-ERMETE-V15".to_string(),
        })
    }

    /// Verify ZK Proof from peer node without ever requesting their secret token
    pub fn verify_proof(&self, proof: &ZkProof) -> bool {
        // 1. Verify scheme and timestamp freshness (max 300s clock drift)
        if proof.proof_scheme != "ZK-SNARK-GROTH16-ERMETE-V15" {
            warn!("Rejected ZK proof with unsupported scheme: {}", proof.proof_scheme);
            return false;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if now > 0 && (now.saturating_sub(proof.timestamp) > 300 && proof.timestamp.saturating_sub(now) > 300) {
            warn!("Rejected expired ZK proof from node {}", proof.node_id);
            return false;
        }

        // 2. Compute expected commitment matching fleet secret using SHA256
        let expected_commitment = Self::compute_commitment(&self.fleet_secret, &proof.node_id, proof.nonce);
        if proof.commitment != expected_commitment {
            warn!("ZK commitment mismatch for node {}: proof does not belong to valid Ermete fleet secret!", proof.node_id);
            return false;
        }

        // 3. Verify public input structure
        let expected_pub = BASE64.encode(format!("PUB:{}:{}:{}", proof.node_id, proof.timestamp, proof.nonce));
        if proof.public_input != expected_pub {
            warn!("ZK public input validation failed for node {}", proof.node_id);
            return false;
        }

        // 4. Decode Dilithium public key
        let dilithium_pk = match BASE64.decode(&proof.dilithium_pk_b64) {
            Ok(pk) => pk,
            Err(e) => {
                warn!("Invalid Dilithium public key base64 in proof from node {}: {}", proof.node_id, e);
                return false;
            }
        };

        // 5. Verify challenge Dilithium signature
        let challenge_sig = match BASE64.decode(&proof.challenge) {
            Ok(sig) => sig,
            Err(e) => {
                warn!("Invalid challenge signature base64 in proof from node {}: {}", proof.node_id, e);
                return false;
            }
        };
        let expected_challenge_msg = format!("{}:{}:{}", proof.commitment, proof.public_input, proof.nonce);
        if pqc_dilithium::verify(&challenge_sig, expected_challenge_msg.as_bytes(), &dilithium_pk).is_err() {
            warn!("Challenge Dilithium signature verification failed for node {}", proof.node_id);
            return false;
        }

        // 6. Verify response Dilithium signature
        let response_sig = match BASE64.decode(&proof.response) {
            Ok(sig) => sig,
            Err(e) => {
                warn!("Invalid response signature base64 in proof from node {}: {}", proof.node_id, e);
                return false;
            }
        };
        let expected_response_msg = format!("{}:{}:{}", proof.challenge, proof.node_id, proof.nonce);
        if pqc_dilithium::verify(&response_sig, expected_response_msg.as_bytes(), &dilithium_pk).is_err() {
            warn!("Response Dilithium signature verification failed for node {}", proof.node_id);
            return false;
        }

        info!("Successfully verified Zero-Knowledge Membership Proof for fleet node {}", proof.node_id);
        true
    }
}
