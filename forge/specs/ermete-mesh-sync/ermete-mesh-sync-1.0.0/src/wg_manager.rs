use tracing::info;
use x25519_dalek::{EphemeralSecret, PublicKey};
use rand_core::OsRng;
use pqc_kyber::{Keypair as KyberKeypair, KYBER_CIPHERTEXTBYTES, KYBER_SSBYTES};
use pqc_dilithium::Keypair as DilithiumKeypair;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

pub struct WgMeshManager {
    public_key: PublicKey,
    kyber_keypair: KyberKeypair,
    dilithium_keypair: DilithiumKeypair,
}

impl WgMeshManager {
    pub fn new() -> Self {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        
        let mut rng = OsRng;
        let kyber_keypair = pqc_kyber::keypair(&mut rng).expect("Failed to generate Kyber-1024 keypair");
        let dilithium_keypair = DilithiumKeypair::generate();

        info!("Initialized WgMeshManager with Hybrid Classical (X25519) + PQC (Kyber-1024 ML-KEM / Dilithium5 ML-DSA) keys.");

        Self {
            public_key: public,
            kyber_keypair,
            dilithium_keypair,
        }
    }

    #[allow(dead_code)]
    pub fn kyber_public_key(&self) -> &pqc_kyber::PublicKey {
        &self.kyber_keypair.public
    }

    #[allow(dead_code)]
    pub fn dilithium_public_key(&self) -> &[u8] {
        &self.dilithium_keypair.public
    }

    /// Encapsulate a shared secret for a peer using peer's Kyber-1024 public key
    #[allow(dead_code)]
    pub fn encapsulate_pqc_secret(
        peer_pk: &pqc_kyber::PublicKey,
    ) -> Result<([u8; KYBER_CIPHERTEXTBYTES], [u8; KYBER_SSBYTES]), String> {
        let mut rng = OsRng;
        pqc_kyber::encapsulate(peer_pk, &mut rng)
            .map_err(|e| format!("Kyber encapsulation error: {:?}", e))
    }

    /// Decapsulate a ciphertext received from a peer using local Kyber-1024 secret key
    #[allow(dead_code)]
    pub fn decapsulate_pqc_secret(
        &self,
        ciphertext: &[u8; KYBER_CIPHERTEXTBYTES],
    ) -> Result<[u8; KYBER_SSBYTES], String> {
        pqc_kyber::decapsulate(ciphertext, &self.kyber_keypair.secret)
            .map_err(|e| format!("Kyber decapsulation error: {:?}", e))
    }

    /// Sign mesh data using local Dilithium5 private key
    #[allow(dead_code)]
    pub fn sign_node_identity(&self, data: &[u8]) -> Vec<u8> {
        self.dilithium_keypair.sign(data).to_vec()
    }

    /// Verify a peer's mesh signature using peer's Dilithium5 public key bytes
    #[allow(dead_code)]
    pub fn verify_node_identity(data: &[u8], sig: &[u8], peer_pk: &[u8]) -> bool {
        pqc_dilithium::verify(sig, data, peer_pk).is_ok()
    }

    pub async fn initialize_tunnel(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing Post-Quantum WireGuard Mesh Tunnel...");
        
        let pk_base64 = BASE64.encode(self.public_key.as_bytes());
        let kyber_pk_base64 = BASE64.encode(&self.kyber_keypair.public);
        let dilithium_pk_base64 = BASE64.encode(&self.dilithium_keypair.public);

        info!("Node X25519 Public Key: {}", pk_base64);
        info!("Node Kyber-1024 ML-KEM Public Key: {}", kyber_pk_base64);
        info!("Node Dilithium5 ML-DSA Public Key: {}", dilithium_pk_base64);

        info!("Level 13 Post-Quantum WireGuard mesh tunnel scaffolding initialized.");
        Ok(())
    }
}
