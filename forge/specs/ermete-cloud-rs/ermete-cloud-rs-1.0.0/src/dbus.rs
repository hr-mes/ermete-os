use zbus::interface;
use tracing::info;
use crate::sync::SyncEngine;
use std::sync::Arc;

pub struct CloudIface {
    pub engine: Arc<SyncEngine>,
}

#[interface(name = "os.ermete.Cloud")]
impl CloudIface {
    /// Syncs local clipboard to trusted peers with Dilithium5 PQC signature
    async fn push_clipboard(&self, content: String) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to push clipboard to cloud with PQC Dilithium5 auth.");
        
        match self.engine.send_clipboard(&content).await {
            Ok(_) => Ok("Clipboard pushed to peers with PQC signature.".into()),
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
}
