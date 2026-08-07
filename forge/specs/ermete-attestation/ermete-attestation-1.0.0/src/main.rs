use std::sync::Arc;
use tracing::info;

use ermete_cvm_attestation::cvm_manager::{run_cvm_dbus_service, CvmManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Ermete OS Confidential Computing Attestation Daemon (CVM Manager)...");

    let config = Default::default();
    let cvm_manager = Arc::new(CvmManager::new(config));

    run_cvm_dbus_service(cvm_manager).await?;

    Ok(())
}
