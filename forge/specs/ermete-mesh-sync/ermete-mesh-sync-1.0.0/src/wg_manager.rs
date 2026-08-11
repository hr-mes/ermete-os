use ermete_bus_api::pqc::PqcKeys;
use tracing::info;

pub struct WgMeshManager {
    pqc_keys: PqcKeys,
}

impl WgMeshManager {
    pub fn new() -> anyhow::Result<Self> {
        let pqc_keys = PqcKeys::new(None)?;
        info!("Initialized WgMeshManager with Hybrid Classical (X25519) + PQC (Kyber-1024 ML-KEM / Dilithium5 ML-DSA) keys.");

        Ok(Self { pqc_keys })
    }

    pub fn kyber_public_key(&self) -> &[u8] {
        &self.pqc_keys.kyber_keypair().public
    }

    pub async fn initialize_tunnel(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing Post-Quantum WireGuard Mesh Tunnel...");

        let pk_base64 = self.pqc_keys.x25519_pk_b64();
        let kyber_pk_base64 = self.pqc_keys.kyber_pk_b64();
        let dilithium_pk_base64 = self.pqc_keys.dilithium_pk_b64();

        info!("Node X25519 Public Key: {}", pk_base64);
        info!("Node Kyber-1024 ML-KEM Public Key: {}", kyber_pk_base64);
        info!("Node Dilithium5 ML-DSA Public Key: {}", dilithium_pk_base64);

        info!("Level 13 Post-Quantum WireGuard mesh tunnel scaffolding initialized.");
        Ok(())
    }
}
