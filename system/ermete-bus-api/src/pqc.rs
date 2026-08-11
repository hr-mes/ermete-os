use anyhow::{anyhow, Result};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use pqc_dilithium::Keypair as DilithiumKeypair;
use pqc_kyber::{Keypair as KyberKeypair, KYBER_CIPHERTEXTBYTES, KYBER_PUBLICKEYBYTES, KYBER_SSBYTES};
use rand::rngs::OsRng;
use ring::hkdf;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{debug, info};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};

use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::NodeIdentityPayload;

/// FIPS 140-3 Zero-Trust Secret Session Key Container in RAM.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretSessionKey {
    pub key: [u8; 32],
}

impl SecretSessionKey {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
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

pub struct HandshakeSession {
    pub ephemeral_secret: EphemeralSecret,
    pub timestamp: u64,
}

impl HandshakeSession {
    pub fn complete_handshake(
        self,
        pqc_keys: &PqcKeys,
        resp: &HandshakeResponsePayload,
        peer_dilithium_pk: &[u8],
    ) -> Result<[u8; 32]> {
        let mut resp_msg = Vec::new();
        resp_msg.extend_from_slice(resp.responder_node_id.as_bytes());
        resp_msg.extend_from_slice(&resp.kyber_ciphertext);
        resp_msg.extend_from_slice(&resp.ephemeral_x25519_pk);
        resp_msg.extend_from_slice(&resp.timestamp.to_le_bytes());

        if !PqcKeys::verify_signature(&resp_msg, &resp.signature, peer_dilithium_pk) {
            return Err(anyhow!(
                "Dilithium5 signature verification failed for response from node {}",
                resp.responder_node_id
            ));
        }

        let mut kyber_ss = pqc_keys.decapsulate_pqc_secret(&resp.kyber_ciphertext)?;
        let peer_x25519_pk = X25519PublicKey::from(resp.ephemeral_x25519_pk);
        let mut x25519_ss = self.ephemeral_secret.diffie_hellman(&peer_x25519_pk).to_bytes();

        let session_key = PqcKeys::derive_session_key(&kyber_ss, &x25519_ss, &resp.timestamp.to_le_bytes())?;
        kyber_ss.zeroize();
        x25519_ss.zeroize();

        Ok(session_key)
    }
}

/// Unified PQC Cryptographic Key Manager for Ermete OS Mesh
#[derive(Clone)]
pub struct PqcKeys {
    inner: Arc<PqcKeysInner>,
}

struct PqcKeysInner {
    x25519_public: X25519PublicKey,
    kyber_keypair: KyberKeypair,
    dilithium_keypair: DilithiumKeypair,
    node_id: String,
}

impl PqcKeys {
    pub fn new(node_id: Option<String>) -> Result<Self> {
        let mut rng = OsRng;

        let secret = EphemeralSecret::random_from_rng(rng);
        let x25519_public = X25519PublicKey::from(&secret);

        let kyber_keypair = pqc_kyber::keypair(&mut rng)
            .map_err(|e| anyhow!("Failed to generate Kyber-1024 ML-KEM keypair: {:?}", e))?;

        let dilithium_keypair = DilithiumKeypair::generate();

        let id = node_id.unwrap_or_else(|| {
            let mut hasher = Sha256::new();
            hasher.update(dilithium_keypair.public);
            format!("node-{}", hex::encode(&hasher.finalize()[..8]))
        });

        info!(
            "PQC Keys initialized for node '{}' (ML-KEM-1024 / Dilithium5)",
            id
        );

        Ok(Self {
            inner: Arc::new(PqcKeysInner {
                x25519_public,
                kyber_keypair,
                dilithium_keypair,
                node_id: id,
            }),
        })
    }

    pub fn node_id(&self) -> &str {
        &self.inner.node_id
    }

    pub fn x25519_public(&self) -> &X25519PublicKey {
        &self.inner.x25519_public
    }

    pub fn kyber_keypair(&self) -> &KyberKeypair {
        &self.inner.kyber_keypair
    }

    pub fn dilithium_keypair(&self) -> &DilithiumKeypair {
        &self.inner.dilithium_keypair
    }

    pub fn x25519_pk_b64(&self) -> String {
        BASE64.encode(self.inner.x25519_public.as_bytes())
    }

    pub fn kyber_pk_b64(&self) -> String {
        BASE64.encode(self.inner.kyber_keypair.public)
    }

    pub fn dilithium_pk_b64(&self) -> String {
        BASE64.encode(self.inner.dilithium_keypair.public)
    }

    pub fn get_node_identity(&self) -> NodeIdentityPayload {
        NodeIdentityPayload {
            node_id: self.inner.node_id.clone(),
            x25519_public_b64: self.x25519_pk_b64(),
            kyber_public_b64: self.kyber_pk_b64(),
            dilithium_public_b64: self.dilithium_pk_b64(),
            pqc_level: "Level 13 (Kyber-1024 / Dilithium5 Zero-Trust)".to_string(),
        }
    }

    pub fn rotate_keys(&mut self) -> Result<()> {
        let mut rng = OsRng;

        let secret = EphemeralSecret::random_from_rng(rng);
        let x25519_public = X25519PublicKey::from(&secret);

        let kyber_keypair = pqc_kyber::keypair(&mut rng)
            .map_err(|e| anyhow!("Failed to generate Kyber-1024 ML-KEM keypair: {:?}", e))?;

        let dilithium_keypair = DilithiumKeypair::generate();

        let node_id = self.inner.node_id.clone();

        self.inner = Arc::new(PqcKeysInner {
            x25519_public,
            kyber_keypair,
            dilithium_keypair,
            node_id,
        });

        info!(
            "PQC Cryptographic keypair rotated successfully for node '{}'",
            self.inner.node_id
        );

        Ok(())
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.inner.dilithium_keypair.sign(data).to_vec()
    }

    pub fn verify_signature(data: &[u8], signature: &[u8], peer_dilithium_pk: &[u8]) -> bool {
        pqc_dilithium::verify(signature, data, peer_dilithium_pk).is_ok()
    }

    pub fn encapsulate_pqc_secret(
        peer_kyber_pk_bytes: &[u8],
    ) -> Result<([u8; KYBER_CIPHERTEXTBYTES], [u8; KYBER_SSBYTES])> {
        if peer_kyber_pk_bytes.len() != KYBER_PUBLICKEYBYTES {
            return Err(anyhow!(
                "Invalid Kyber-1024 public key length: expected {}, got {}",
                KYBER_PUBLICKEYBYTES,
                peer_kyber_pk_bytes.len()
            ));
        }
        let mut pk_arr = [0u8; KYBER_PUBLICKEYBYTES];
        pk_arr.copy_from_slice(peer_kyber_pk_bytes);
        let peer_pk = pqc_kyber::PublicKey::from(pk_arr);

        let mut rng = OsRng;
        pqc_kyber::encapsulate(&peer_pk, &mut rng)
            .map_err(|e| anyhow!("Kyber-1024 encapsulation failed: {:?}", e))
    }

    pub fn decapsulate_pqc_secret(
        &self,
        ciphertext_bytes: &[u8],
    ) -> Result<[u8; KYBER_SSBYTES]> {
        if ciphertext_bytes.len() != KYBER_CIPHERTEXTBYTES {
            return Err(anyhow!(
                "Invalid Kyber-1024 ciphertext length: expected {}, got {}",
                KYBER_CIPHERTEXTBYTES,
                ciphertext_bytes.len()
            ));
        }
        let mut ct_arr = [0u8; KYBER_CIPHERTEXTBYTES];
        ct_arr.copy_from_slice(ciphertext_bytes);

        pqc_kyber::decapsulate(&ct_arr, &self.inner.kyber_keypair.secret)
            .map_err(|e| anyhow!("Kyber-1024 decapsulation failed: {:?}", e))
    }

    pub fn derive_session_key(
        kyber_ss: &[u8; KYBER_SSBYTES],
        x25519_ss: &[u8; 32],
        salt: &[u8],
    ) -> Result<[u8; 32]> {
        let mut ikm = [0u8; 64];
        ikm[..KYBER_SSBYTES].copy_from_slice(kyber_ss);
        ikm[KYBER_SSBYTES..64].copy_from_slice(x25519_ss);

        let salt_obj = hkdf::Salt::new(hkdf::HKDF_SHA256, salt);
        let prk = salt_obj.extract(&ikm);
        ikm.zeroize();
        let okm = prk
            .expand(&[b"ermete-mesh-bus-pqc-v1-session"], hkdf::HKDF_SHA256)
            .map_err(|e| anyhow!("HKDF expansion failed: {:?}", e))?;

        let mut session_key = [0u8; 32];
        okm.fill(&mut session_key).map_err(|e| anyhow!("Fill session key failed: {:?}", e))?;
        Ok(session_key)
    }

    pub fn build_handshake_init(&self, timestamp: u64) -> (HandshakeInitPayload, HandshakeSession) {
        let eph_secret = EphemeralSecret::random_from_rng(OsRng);
        let eph_public = X25519PublicKey::from(&eph_secret);
        let eph_bytes = *eph_public.as_bytes();

        let mut msg_to_sign = Vec::new();
        msg_to_sign.extend_from_slice(self.inner.node_id.as_bytes());
        msg_to_sign.extend_from_slice(&eph_bytes);
        msg_to_sign.extend_from_slice(&self.inner.kyber_keypair.public);
        msg_to_sign.extend_from_slice(&timestamp.to_le_bytes());

        let signature = self.sign(&msg_to_sign);

        let payload = HandshakeInitPayload {
            sender_node_id: self.inner.node_id.clone(),
            ephemeral_x25519_pk: eph_bytes,
            kyber_pk: self.inner.kyber_keypair.public.to_vec(),
            timestamp,
            signature,
        };

        let session = HandshakeSession {
            ephemeral_secret: eph_secret,
            timestamp,
        };

        (payload, session)
    }

    pub fn process_handshake_init(
        &self,
        init: &HandshakeInitPayload,
        peer_dilithium_pk: &[u8],
        timestamp: u64,
    ) -> Result<(HandshakeResponsePayload, [u8; 32])> {
        let mut msg = Vec::new();
        msg.extend_from_slice(init.sender_node_id.as_bytes());
        msg.extend_from_slice(&init.ephemeral_x25519_pk);
        msg.extend_from_slice(&init.kyber_pk);
        msg.extend_from_slice(&init.timestamp.to_le_bytes());

        if !Self::verify_signature(&msg, &init.signature, peer_dilithium_pk) {
            return Err(anyhow!(
                "Handshake Init Dilithium5 signature verification failed for node {}",
                init.sender_node_id
            ));
        }

        let (ct, mut kyber_ss) = Self::encapsulate_pqc_secret(&init.kyber_pk)?;

        let eph_resp = EphemeralSecret::random_from_rng(OsRng);
        let eph_resp_pk = X25519PublicKey::from(&eph_resp);
        let peer_x25519_pk = X25519PublicKey::from(init.ephemeral_x25519_pk);
        let mut x25519_ss = eph_resp.diffie_hellman(&peer_x25519_pk).to_bytes();

        let session_key = Self::derive_session_key(&kyber_ss, &x25519_ss, &init.timestamp.to_le_bytes())?;
        kyber_ss.zeroize();
        x25519_ss.zeroize();

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
