use anyhow::Result;
pub use ermete_bus_api::pqc::{
    HandshakeInitPayload, HandshakeResponsePayload, HandshakeSession, PqcKeys, SecretSessionKey,
};
pub use ermete_bus_api::NodeIdentityPayload as NodeIdentity;
use pqc_kyber::{KYBER_CIPHERTEXTBYTES, KYBER_SSBYTES};
use ring::aead;

pub fn decrypt_aes_gcm(session_key: &SecretSessionKey, payload: &[u8]) -> Result<Vec<u8>> {
    if payload.len() < 12 {
        return Err(anyhow::anyhow!("Payload too short for AES-GCM nonce"));
    }
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, session_key.as_bytes())
        .map_err(|_| anyhow::anyhow!("Invalid AEAD key"))?;
    let key = aead::LessSafeKey::new(unbound_key);
    let nonce = aead::Nonce::try_assume_unique_for_key(&payload[..12])
        .map_err(|_| anyhow::anyhow!("Invalid nonce"))?;
    let mut in_out = payload[12..].to_vec();
    let decrypted = key.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|_| anyhow::anyhow!("AEAD decryption failed"))?;
    Ok(decrypted.to_vec())
}

/// Level 13 Post-Quantum Cryptographic Engine for Zero-Trust Mesh
/// Delegating core cryptographic key logic to unified `ermete_bus_api::pqc::PqcKeys`
#[derive(Clone)]
pub struct PqcEngine {
    keys: PqcKeys,
}

impl PqcEngine {
    pub fn new(node_id: Option<String>) -> Result<Self> {
        let keys = PqcKeys::new(node_id)?;
        Ok(Self { keys })
    }

    pub fn keys(&self) -> &PqcKeys {
        &self.keys
    }

    pub fn node_id(&self) -> &str {
        self.keys.node_id()
    }

    pub fn get_node_identity(&self) -> NodeIdentity {
        self.keys.get_node_identity()
    }

    pub fn rotate_keys(&mut self) -> Result<()> {
        self.keys.rotate_keys()
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.keys.sign(data)
    }

    pub fn verify_signature(data: &[u8], signature: &[u8], peer_dilithium_pk: &[u8]) -> bool {
        PqcKeys::verify_signature(data, signature, peer_dilithium_pk)
    }

    pub fn encapsulate_pqc_secret(
        peer_kyber_pk_bytes: &[u8],
    ) -> Result<([u8; KYBER_CIPHERTEXTBYTES], [u8; KYBER_SSBYTES])> {
        PqcKeys::encapsulate_pqc_secret(peer_kyber_pk_bytes)
    }

    pub fn decapsulate_pqc_secret(
        &self,
        ciphertext_bytes: &[u8],
    ) -> Result<[u8; KYBER_SSBYTES]> {
        self.keys.decapsulate_pqc_secret(ciphertext_bytes)
    }

    pub fn derive_session_key(
        kyber_ss: &[u8; KYBER_SSBYTES],
        x25519_ss: &[u8; 32],
        salt: &[u8],
    ) -> Result<[u8; 32]> {
        PqcKeys::derive_session_key(kyber_ss, x25519_ss, salt)
    }

    pub fn build_handshake_init(&self, timestamp: u64) -> (HandshakeInitPayload, HandshakeSession) {
        self.keys.build_handshake_init(timestamp)
    }

    pub fn process_handshake_init(
        &self,
        init: &HandshakeInitPayload,
        peer_dilithium_pk: &[u8],
        timestamp: u64,
    ) -> Result<(HandshakeResponsePayload, [u8; 32])> {
        self.keys.process_handshake_init(init, peer_dilithium_pk, timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD as BASE64;

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
    fn test_pqc_engine_rotate_keys() -> Result<()> {
        let mut engine = PqcEngine::new(Some("test-node-rotate".to_string()))?;
        let identity_before = engine.get_node_identity();

        engine.rotate_keys()?;
        let identity_after = engine.get_node_identity();

        assert_eq!(identity_before.node_id, identity_after.node_id);
        assert_ne!(identity_before.kyber_public_b64, identity_after.kyber_public_b64);
        assert_ne!(identity_before.dilithium_public_b64, identity_after.dilithium_public_b64);
        assert_ne!(identity_before.x25519_public_b64, identity_after.x25519_public_b64);

        Ok(())
    }

    #[test]
    fn test_pqc_handshake_flow() -> Result<()> {
        let alice = PqcEngine::new(Some("alice".to_string()))?;
        let bob = PqcEngine::new(Some("bob".to_string()))?;

        let alice_id = alice.get_node_identity();
        let alice_dilithium_pk = BASE64.decode(&alice_id.dilithium_public_b64)?;

        let bob_id = bob.get_node_identity();
        let bob_dilithium_pk = BASE64.decode(&bob_id.dilithium_public_b64)?;

        let timestamp = 1700000000;
        let (init, session) = alice.build_handshake_init(timestamp);

        let (resp, bob_session_key) = bob.process_handshake_init(&init, &alice_dilithium_pk, timestamp)?;

        let alice_session_key = session.complete_handshake(alice.keys(), &resp, &bob_dilithium_pk)?;

        assert_eq!(alice_session_key, bob_session_key, "Derived session keys between Alice and Bob must match");

        Ok(())
    }
}
