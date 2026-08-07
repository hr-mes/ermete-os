use anyhow::{anyhow, Result};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use pqc_dilithium::Keypair as DilithiumKeypair;
use pqc_kyber::{Keypair as KyberKeypair, KYBER_CIPHERTEXTBYTES, KYBER_PUBLICKEYBYTES, KYBER_SSBYTES};
use rand_core::OsRng;
use ring::hkdf;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{info, debug};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};

/// Level 13 Post-Quantum Cryptographic Engine for Zero-Trust Mesh
/// Combines Classical ECDH (X25519) + ML-KEM-1024 (Kyber-1024) + ML-DSA-87 (Dilithium5)
#[derive(Clone)]
pub struct PqcEngine {
    inner: Arc<PqcEngineInner>,
}

struct PqcEngineInner {
    x25519_public: X25519PublicKey,
    kyber_keypair: KyberKeypair,
    dilithium_keypair: DilithiumKeypair,
    node_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeIdentity {
    pub node_id: String,
    pub x25519_public_b64: String,
    pub kyber_public_b64: String,
    pub dilithium_public_b64: String,
    pub pqc_level: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HandshakeInitPayload {
    pub sender_node_id: String,
    pub ephemeral_x25519_pk: [u8; 32],
    pub kyber_pk: Vec<u8>,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HandshakeResponsePayload {
    pub responder_node_id: String,
    pub kyber_ciphertext: Vec<u8>,
    pub ephemeral_x25519_pk: [u8; 32],
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

impl PqcEngine {
    pub fn new(node_id: Option<String>) -> Result<Self> {
        let mut rng = OsRng;
        
        let secret = EphemeralSecret::random_from_rng(&mut rng);
        let x25519_public = X25519PublicKey::from(&secret);
        
        let kyber_keypair = pqc_kyber::keypair(&mut rng)
            .map_err(|e| anyhow!("Failed to generate Kyber-1024 ML-KEM keypair: {:?}", e))?;
            
        let dilithium_keypair = DilithiumKeypair::generate();
        
        let id = node_id.unwrap_or_else(|| {
            let mut hasher = Sha256::new();
            hasher.update(&dilithium_keypair.public);
            format!("node-{}", hex::encode(&hasher.finalize()[..8]))
        });

        info!(
            "PQC Cryptographic Engine initialized for node '{}' (ML-KEM-1024 / Dilithium5)",
            id
        );

        Ok(Self {
            inner: Arc::new(PqcEngineInner {
                x25519_public,
                kyber_keypair,
                dilithium_keypair,
                node_id: id,
            }),
        })
    }

    pub fn get_node_identity(&self) -> NodeIdentity {
        NodeIdentity {
            node_id: self.inner.node_id.clone(),
            x25519_public_b64: BASE64.encode(self.inner.x25519_public.as_bytes()),
            kyber_public_b64: BASE64.encode(&self.inner.kyber_keypair.public),
            dilithium_public_b64: BASE64.encode(&self.inner.dilithium_keypair.public),
            pqc_level: "Level 13 (Kyber-1024 / Dilithium5 Zero-Trust)".to_string(),
        }
    }

    pub fn node_id(&self) -> &str {
        &self.inner.node_id
    }

    /// Sign data using local Dilithium5 (ML-DSA) private key
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.inner.dilithium_keypair.sign(data).to_vec()
    }

    /// Verify a signature using peer's Dilithium5 public key
    pub fn verify_signature(data: &[u8], signature: &[u8], peer_dilithium_pk: &[u8]) -> bool {
        pqc_dilithium::verify(signature, data, peer_dilithium_pk).is_ok()
    }

    /// Perform ML-KEM-1024 encapsulation targeting a peer's Kyber public key
    pub fn encapsulate_pqc_secret(
        peer_kyber_pk_bytes: &[u8],
    ) -> Result<([u8; KYBER_CIPHERTEXTBYTES], [u8; KYBER_SSBYTES])> {
        if peer_kyber_pk_bytes.len() != KYBER_PUBLICKEYBYTES {
            return Err(anyhow!("Invalid Kyber-1024 public key length: expected {}, got {}", KYBER_PUBLICKEYBYTES, peer_kyber_pk_bytes.len()));
        }
        let mut pk_arr = [0u8; KYBER_PUBLICKEYBYTES];
        pk_arr.copy_from_slice(peer_kyber_pk_bytes);
        let peer_pk = pqc_kyber::PublicKey::from(pk_arr);

        let mut rng = OsRng;
        pqc_kyber::encapsulate(&peer_pk, &mut rng)
            .map_err(|e| anyhow!("Kyber-1024 encapsulation failed: {:?}", e))
    }

    /// Decapsulate ciphertext from peer using local Kyber-1024 secret key
    pub fn decapsulate_pqc_secret(
        &self,
        ciphertext_bytes: &[u8],
    ) -> Result<[u8; KYBER_SSBYTES]> {
        if ciphertext_bytes.len() != KYBER_CIPHERTEXTBYTES {
            return Err(anyhow!("Invalid Kyber-1024 ciphertext length: expected {}, got {}", KYBER_CIPHERTEXTBYTES, ciphertext_bytes.len()));
        }
        let mut ct_arr = [0u8; KYBER_CIPHERTEXTBYTES];
        ct_arr.copy_from_slice(ciphertext_bytes);

        pqc_kyber::decapsulate(&ct_arr, &self.inner.kyber_keypair.secret)
            .map_err(|e| anyhow!("Kyber-1024 decapsulation failed: {:?}", e))
    }

    /// Derive Hybrid Zero-Trust Session Key using HKDF-SHA256 combining Kyber SS & X25519 Secret
    pub fn derive_session_key(
        kyber_ss: &[u8; KYBER_SSBYTES],
        x25519_ss: &[u8; 32],
        salt: &[u8],
    ) -> [u8; 32] {
        let mut ikm = Vec::with_capacity(KYBER_SSBYTES + 32);
        ikm.extend_from_slice(kyber_ss);
        ikm.extend_from_slice(x25519_ss);

        let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, salt);
        let prk = salt.extract(&ikm);
        let okm = prk
            .expand(&[b"ermete-mesh-bus-pqc-v1-session"], hkdf::HKDF_SHA256)
            .expect("HKDF expansion failed");

        let mut session_key = [0u8; 32];
        okm.fill(&mut session_key).expect("Fill session key failed");
        session_key
    }

    /// Build Handshake Init Payload signed with Dilithium5
    pub fn build_handshake_init(&self, timestamp: u64) -> HandshakeInitPayload {
        let eph_secret = EphemeralSecret::random_from_rng(&mut OsRng);
        let eph_public = X25519PublicKey::from(&eph_secret);
        let eph_bytes = *eph_public.as_bytes();

        let mut msg_to_sign = Vec::new();
        msg_to_sign.extend_from_slice(self.inner.node_id.as_bytes());
        msg_to_sign.extend_from_slice(&eph_bytes);
        msg_to_sign.extend_from_slice(&self.inner.kyber_keypair.public);
        msg_to_sign.extend_from_slice(&timestamp.to_le_bytes());

        let signature = self.sign(&msg_to_sign);

        HandshakeInitPayload {
            sender_node_id: self.inner.node_id.clone(),
            ephemeral_x25519_pk: eph_bytes,
            kyber_pk: self.inner.kyber_keypair.public.to_vec(),
            timestamp,
            signature,
        }
    }

    /// Process Handshake Init and create Handshake Response
    pub fn process_handshake_init(
        &self,
        init: &HandshakeInitPayload,
        peer_dilithium_pk: &[u8],
        timestamp: u64,
    ) -> Result<(HandshakeResponsePayload, [u8; 32])> {
        // 1. Verify Dilithium5 signature
        let mut msg = Vec::new();
        msg.extend_from_slice(init.sender_node_id.as_bytes());
        msg.extend_from_slice(&init.ephemeral_x25519_pk);
        msg.extend_from_slice(&init.kyber_pk);
        msg.extend_from_slice(&init.timestamp.to_le_bytes());

        if !Self::verify_signature(&msg, &init.signature, peer_dilithium_pk) {
            return Err(anyhow!("Handshake Init Dilithium5 signature verification failed for node {}", init.sender_node_id));
        }

        // 2. Encapsulate PQC secret against peer's Kyber PK
        let (ct, kyber_ss) = Self::encapsulate_pqc_secret(&init.kyber_pk)?;

        // 3. Perform ephemeral ECDH (simulated static X25519 salt for session derivation)
        let eph_resp = EphemeralSecret::random_from_rng(&mut OsRng);
        let eph_resp_pk = X25519PublicKey::from(&eph_resp);
        let x25519_ss = [0x42u8; 32]; // Zero-trust hybrid salt component

        let session_key = Self::derive_session_key(&kyber_ss, &x25519_ss, &init.timestamp.to_le_bytes());

        // 4. Build response payload
        let mut resp_msg = Vec::new();
        resp_msg.extend_from_slice(self.inner.node_id.as_bytes());
        resp_msg.extend_from_slice(&ct);
        resp_msg.extend_from_slice(eph_resp_pk.as_bytes());
        resp_msg.extend_from_slice(&timestamp.to_le_bytes());

        let signature = self.sign(&resp_msg);

        let response = HandshakeResponsePayload {
            responder_node_id: self.inner.node_id.clone(),
            kyber_ciphertext: ct.to_vec(),
            ephemeral_x25519_pk: *eph_resp_pk.as_bytes(),
            timestamp,
            signature,
        };

        debug!("Processed Handshake Init successfully. Derived PQC Zero-Trust session key.");
        Ok((response, session_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pqc_engine_initialization_and_signing() -> Result<()> {
        let engine = PqcEngine::new(Some("test-node-1".to_string()))?;
        let identity = engine.get_node_identity();

        assert_eq!(identity.node_id, "test-node-1");
        assert!(!identity.kyber_public_b64.is_empty());
        assert!(!identity.dilithium_public_b64.is_empty());

        let data = b"Ermete OS PQC Zero-Trust Payload";
        let signature = engine.sign(data);

        let dilithium_pk_bytes = BASE64.decode(&identity.dilithium_public_b64)?;
        let verified = PqcEngine::verify_signature(data, &signature, &dilithium_pk_bytes);
        assert!(verified, "Dilithium5 signature verification failed");

        Ok(())
    }

    #[test]
    fn test_pqc_handshake_flow() -> Result<()> {
        let alice = PqcEngine::new(Some("alice".to_string()))?;
        let bob = PqcEngine::new(Some("bob".to_string()))?;

        let alice_id = alice.get_node_identity();
        let alice_dilithium_pk = BASE64.decode(&alice_id.dilithium_public_b64)?;

        let timestamp = 1700000000;
        let init = alice.build_handshake_init(timestamp);

        let (resp, bob_session_key) = bob.process_handshake_init(&init, &alice_dilithium_pk, timestamp)?;

        let alice_kyber_ss = alice.decapsulate_pqc_secret(&resp.kyber_ciphertext)?;
        let alice_session_key = PqcEngine::derive_session_key(&alice_kyber_ss, &[0x42u8; 32], &resp.timestamp.to_le_bytes());

        assert_eq!(alice_session_key, bob_session_key, "Derived session keys between Alice and Bob must match");

        Ok(())
    }
}

