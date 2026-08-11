pub mod drm_lease;
pub mod model_loader;
pub mod npu;
pub mod security;
pub mod types;

use model_loader::NeuralModelEngine;
use types::{AiIntent, WorkloadClassificationRequest, WorkloadClassificationResponse};

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info};
use zbus::{interface, Connection};

use npu::HardwareOffloader;


pub struct AiDaemonProxy {
    offloader: Arc<HardwareOffloader>,
    model_engine: Arc<RwLock<NeuralModelEngine>>,
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

    /// Real Candle-accelerated workload classification IPC endpoint
    async fn classify_workload(&self, json_query: &str) -> Result<String, zbus::fdo::Error> {
        info!("Received AI Workload Classification Request: {}", json_query);
        let req: WorkloadClassificationRequest = serde_json::from_str(json_query)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Invalid payload format: {}", e)))?;

        let features = vec![req.norm_cpu, req.norm_io, req.norm_mem, req.norm_threads];

        let engine = self.model_engine.read().await;
        let logits = engine
            .infer(&features)
            .map_err(|e| zbus::fdo::Error::Failed(e))?;

        let mut max_idx = 3;
        let mut max_val = f32::NEG_INFINITY;
        for (i, &val) in logits.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = i;
            }
        }

        let category = match max_idx {
            0 => "InteractiveUi",
            1 => "RealtimeNpu",
            2 => "BatchCompute",
            _ => "IdleBackground",
        };

        let response = WorkloadClassificationResponse {
            category: category.to_string(),
            logits,
        };

        serde_json::to_string(&response)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialization error: {}", e)))
    }

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    security::apply_ai_hardening();
    info!("Ermete AI Daemon starting (NPU & Candle Accelerated - 0% CPU Target)...");


    let offloader = Arc::new(HardwareOffloader::new());

    // Initialize Candle real model engine with default weights path
    let default_weights_path = "/etc/ermete/ai/workload_classifier.safetensors";
    let model_engine = Arc::new(RwLock::new(NeuralModelEngine::new(default_weights_path)));

    // Acquire exclusive DRM Lease for AI Offloading
    if let Err(e) = drm_lease::acquire_drm_lease().await {
        error!("Failed to acquire DRM Lease: {}. Falling back to normal mode.", e);
    }

    let hw_info = offloader.get_active_hardware_info();
    info!(
        "Active Hardware Device: backend={:?}, device='{}', CPU target impact={:.1}%",
        hw_info.backend, hw_info.device_name, hw_info.cpu_impact_percentage
    );

    let proxy = AiDaemonProxy {
        offloader,
        model_engine,
    };

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

