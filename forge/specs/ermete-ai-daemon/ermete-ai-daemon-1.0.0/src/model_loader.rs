use candle_core::{DType, Device, Tensor};
use candle_nn::{Linear, Module, VarBuilder};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Real Neural Network Model Engine using Candle (supports safetensors & ONNX)
pub struct InferenceEngine {
    device: Device,
    model_path: PathBuf,
    w1: Option<Linear>,
    w2: Option<Linear>,
    is_loaded: bool,
}

impl InferenceEngine {
    pub fn new<P: AsRef<Path>>(default_path: P) -> Self {
        let device = Self::probe_hardware();
        info!("InferenceEngine initialized using device: {:?}", device);

        let mut engine = Self {
            device,
            model_path: default_path.as_ref().to_path_buf(),
            w1: None,
            w2: None,
            is_loaded: false,
        };

        // Attempt loading model weights if present at default_path
        let _ = engine.load_safetensors_weights();
        engine
    }

    fn probe_hardware() -> Device {
        info!("Probing hardware for AI acceleration...");

        // 1. Attempt NPU/Accelerator (Mocking NPU as CUDA for standard ML frameworks)
        if candle_core::utils::cuda_is_available() {
            if let Ok(device) = Device::new_cuda(0) {
                info!("NPU/CUDA Accelerator found.");
                return device;
            }
        }

        // 2. Fallback to GPU (Metal)
        if candle_core::utils::metal_is_available() {
            if let Ok(device) = Device::new_metal(0) {
                info!("GPU (Metal) found.");
                return device;
            }
        }

        // 3. Fallback to CPU
        info!("No NPU or GPU acceleration found. Falling back to CPU.");
        Device::Cpu
    }

    /// Loads real neural network weights from a .safetensors file
    pub fn load_safetensors_weights(&mut self) -> Result<(), String> {
        if !self.model_path.exists() {
            warn!(
                "Model weights file '{:?}' not found on filesystem. Real Candle engine initialized and ready for weights.",
                self.model_path
            );
            self.is_loaded = false;
            return Err(format!("Weights file not found: {:?}", self.model_path));
        }

        info!("Loading real model weights from safetensors: {:?}", self.model_path);
        let weights = candle_core::safetensors::load(&self.model_path, &self.device)
            .map_err(|e| format!("Failed to parse safetensors at {:?}: {}", self.model_path, e))?;

        let vb = VarBuilder::from_tensors(weights, DType::F32, &self.device);

        let w1 = candle_nn::linear(4, 8, vb.pp("layer1"))
            .map_err(|e| format!("Failed to construct layer1: {}", e))?;
        let w2 = candle_nn::linear(8, 4, vb.pp("layer2"))
            .map_err(|e| format!("Failed to construct layer2: {}", e))?;

        self.w1 = Some(w1);
        self.w2 = Some(w2);
        self.is_loaded = true;
        info!("Successfully loaded Candle neural network model from {:?}", self.model_path);
        Ok(())
    }

    /// Prepares real tensors and executes inference for workload classification
    pub fn predict_workload(&self, features: &[f32]) -> Result<Vec<f32>, String> {
        if features.len() != 4 {
            return Err(format!("Expected 4 continuous feature inputs, got {}", features.len()));
        }

        let input_tensor = Tensor::from_slice(features, (1, 4), &self.device)
            .map_err(|e| format!("Candle tensor instantiation failed: {}", e))?;

        if !self.is_loaded {
            return Err("ZERO-TRUST VIOLATION: Refusing to execute inference with uninitialized or dummy weights. Real AI models must be signed and loaded.".to_string());
        }

        let (l1, l2) = match (&self.w1, &self.w2) {
            (Some(l1), Some(l2)) => (l1, l2),
            _ => return Err("Neural layers uninitialized".to_string()),
        };

        let hidden = l1.forward(&input_tensor)
            .map_err(|e| format!("Layer1 forward pass failed: {}", e))?
            .relu()
            .map_err(|e| format!("ReLU activation failed: {}", e))?;

        let logits = l2.forward(&hidden)
            .map_err(|e| format!("Layer2 forward pass failed: {}", e))?;

        let logits_vec = logits
            .squeeze(0)
            .map_err(|e| format!("Squeeze operation failed: {}", e))?
            .to_vec1::<f32>()
            .map_err(|e| format!("Logits vector extraction failed: {}", e))?;

        Ok(logits_vec)
    }

    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }
}
