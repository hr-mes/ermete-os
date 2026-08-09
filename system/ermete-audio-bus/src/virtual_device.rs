use crate::pipewire_manager::PipewireManager;
use std::sync::Arc;

/// Micro-service responsible for creating and managing virtual audio devices.
#[derive(Clone)]
pub struct AudioVirtualDeviceService {
    pw_manager: Arc<PipewireManager>,
}

impl AudioVirtualDeviceService {
    pub fn new(pw_manager: Arc<PipewireManager>) -> Self {
        Self { pw_manager }
    }

    pub async fn create_virtual_sink(&self, name: String, channels: u32) -> String {
        match self.pw_manager.create_virtual_sink(name, channels).await {
            Ok(id) => format!("Created virtual sink with ID {}", id),
            Err(e) => format!("Error creating virtual sink: {}", e),
        }
    }
}
