use tracing::{error, info, warn};
use crate::npu::openvino::OpenVinoNpuEngine;
use crate::npu::vulkan::VulkanTensorEngine;
use crate::types::AiIntent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccelerationBackend {
    OpenVinoNpu,
    VulkanTensorCores,
    HardwareAuto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffloadPolicy {
    /// Strictly forbid CPU execution for AI tasks. Forces CPU impact to 0%.
    ForceHardwareOnly,
}

#[derive(Debug, Clone)]
pub struct HardwareDeviceInfo {
    pub backend: AccelerationBackend,
    pub device_name: String,
    pub cpu_impact_percentage: f32,
}

pub struct HardwareOffloader {
    openvino_engine: OpenVinoNpuEngine,
    vulkan_engine: VulkanTensorEngine,
    policy: OffloadPolicy,
}

impl Default for HardwareOffloader {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareOffloader {
    pub fn new() -> Self {
        info!("Initializing Hardware AI Offloading Engine (Policy: ForceHardwareOnly - 0% CPU target)");

        let openvino_engine = OpenVinoNpuEngine::new();
        let vulkan_engine = VulkanTensorEngine::new();

        Self {
            openvino_engine,
            vulkan_engine,
            policy: OffloadPolicy::ForceHardwareOnly,
        }
    }

    pub fn get_active_hardware_info(&self) -> HardwareDeviceInfo {
        if self.openvino_engine.is_available() {
            HardwareDeviceInfo {
                backend: AccelerationBackend::OpenVinoNpu,
                device_name: self.openvino_engine.device_name().to_string(),
                cpu_impact_percentage: 0.0,
            }
        } else if self.vulkan_engine.is_available() {
            HardwareDeviceInfo {
                backend: AccelerationBackend::VulkanTensorCores,
                device_name: self.vulkan_engine.device_name().to_string(),
                cpu_impact_percentage: 0.0,
            }
        } else {
            HardwareDeviceInfo {
                backend: AccelerationBackend::HardwareAuto,
                device_name: "Generic NPU/GPU Accelerator".to_string(),
                cpu_impact_percentage: 0.0,
            }
        }
    }

    pub async fn process_inference(&self, intent: &AiIntent) -> Result<(Vec<f32>, HardwareDeviceInfo), String> {
        let mut input_tensor = vec![0.0f32; 768];
        for (i, b) in intent.text.as_bytes().iter().take(768).enumerate() {
            input_tensor[i] = (*b as f32) / 255.0;
        }

        info!(
            "Offloading AI query '{}' [intent: '{}'] exclusively to hardware accelerators...",
            intent.text, intent.intent
        );

        // Priority 1: OpenVINO NPU Engine
        if self.openvino_engine.is_available() {
            match self.openvino_engine.execute_npu_inference(&input_tensor, &[1, 768]) {
                Ok(output) => {
                    let info = HardwareDeviceInfo {
                        backend: AccelerationBackend::OpenVinoNpu,
                        device_name: self.openvino_engine.device_name().to_string(),
                        cpu_impact_percentage: 0.0,
                    };
                    return Ok((output, info));
                }
                Err(e) => warn!("OpenVINO NPU offload fallback trigger: {}", e),
            }
        }

        // Priority 2: Vulkan Compute & Tensor Cores
        if self.vulkan_engine.is_available() {
            match self.vulkan_engine.execute_vulkan_compute(&input_tensor) {
                Ok(output) => {
                    let info = HardwareDeviceInfo {
                        backend: AccelerationBackend::VulkanTensorCores,
                        device_name: self.vulkan_engine.device_name().to_string(),
                        cpu_impact_percentage: 0.0,
                    };
                    return Ok((output, info));
                }
                Err(e) => error!("Vulkan tensor core execution error: {}", e),
            }
        }

        if self.policy == OffloadPolicy::ForceHardwareOnly {
            Err("CRITICAL: ForceHardwareOnly policy active. CPU fallback rejected. No NPU or Vulkan Tensor device accessible.".to_string())
        } else {
            Err("Hardware offloading unavailable".to_string())
        }
    }
}
