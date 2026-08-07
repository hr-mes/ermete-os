pub mod drm_lease;
pub mod npu;
pub mod types;
use types::AiIntent;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use zbus::{interface, Connection};

use npu::HardwareOffloader;


pub struct AiDaemonProxy {
    offloader: Arc<HardwareOffloader>,
}

#[interface(name = "os.ermete.AiDaemon")]
impl AiDaemonProxy {
    async fn process_query(&self, json_query: &str) -> String {
        info!("Received AI Query: {}", json_query);
        if let Ok(query) = serde_json::from_str::<AiIntent>(json_query) {
            let offloader = self.offloader.clone();

            // Offload inference exclusively to NPU or Vulkan Tensor Cores (0% CPU impact)
            match offloader.process_inference(&query).await {
                Ok((output, hw_info)) => {
                    info!(
                        "Hardware inference succeeded on backend {:?} ('{}'). Output shape: [1, 4]",
                        hw_info.backend, hw_info.device_name
                    );
                    let response = format!(
                        "Processed intent '{}' via Hardware Acceleration Backend '{:?}' on device '{}' [CPU Impact: {:.1}%] -> prediction: {:?}",
                        query.intent, hw_info.backend, hw_info.device_name, hw_info.cpu_impact_percentage, output
                    );
                    info!("Returning: {}", response);
                    response
                }
                Err(e) => {
                    error!("Hardware offloaded inference failed: {}", e);
                    format!("Error: Hardware Offloading Failed ({})", e)
                }
            }
        } else {
            error!("Failed to parse AiIntent");
            "Error: Invalid payload".to_string()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Ermete AI Daemon starting (NPU & Vulkan GPU Hardware Accelerated - 0% CPU Target)...");

    let offloader = Arc::new(HardwareOffloader::new());
    
    // Acquire exclusive DRM Lease for AI Offloading
    if let Err(e) = drm_lease::acquire_drm_lease().await {
        error!("Failed to acquire DRM Lease: {}. Falling back to normal mode.", e);
    }
    
    let hw_info = offloader.get_active_hardware_info();
    info!(
        "Active Hardware Device: backend={:?}, device='{}', CPU target impact={:.1}%",
        hw_info.backend, hw_info.device_name, hw_info.cpu_impact_percentage
    );

    let proxy = AiDaemonProxy { offloader };

    let _conn = Connection::session()
        .await?
        .object_server()
        .at("/os/ermete/AiDaemon", proxy)
        .await?;

    info!("Listening on DBus: os.ermete.AiDaemon");

    // Async event loop
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
