use x25519_dalek::{StaticSecret, PublicKey};
use rand_core::OsRng;
use base64::{Engine as _, engine::general_purpose};

pub struct KyberKeyPair {
    pub public: Vec<u8>,
}

pub struct PqcKeys {
    x25519_sk: [u8; 32],
    x25519_pk: PublicKey,
    kyber_kp: KyberKeyPair,
}
impl PqcKeys {
    pub fn new(_: Option<()>) -> anyhow::Result<Self> {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        Ok(Self {
            x25519_sk: secret.to_bytes(),
            x25519_pk: public,
            kyber_kp: KyberKeyPair { public: vec![0; 32] },
        })
    }
    pub fn x25519_sk(&self) -> &[u8; 32] {
        &self.x25519_sk
    }
    pub fn x25519_pk_b64(&self) -> String {
        general_purpose::STANDARD.encode(self.x25519_pk.as_bytes())
    }
    pub fn kyber_keypair(&self) -> &KyberKeyPair {
        &self.kyber_kp
    }
    pub fn kyber_pk_b64(&self) -> String {
        general_purpose::STANDARD.encode(&self.kyber_kp.public)
    }
    pub fn dilithium_pk_b64(&self) -> String {
        general_purpose::STANDARD.encode(vec![0; 32])
    }
}
