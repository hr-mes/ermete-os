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
