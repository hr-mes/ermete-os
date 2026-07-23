use zbus::interface;
use tracing::info;
use crate::sync::SyncEngine;
use std::sync::Arc;

pub struct CloudIface {
    pub engine: Arc<SyncEngine>,
}

#[interface(name = "os.ermete.Cloud")]
impl CloudIface {
    /// Syncs local clipboard to trusted peers
    async fn push_clipboard(&self, content: String) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to push clipboard to cloud.");
        
        match self.engine.send_clipboard(&content).await {
            Ok(_) => Ok("Clipboard pushed to peers.".into()),
            Err(e) => Ok(format!("Error: {}", e)),
        }
    }
}
