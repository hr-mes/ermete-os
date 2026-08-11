use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AiIntent {
    pub text: String,
    pub intent: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkloadClassificationRequest {
    pub pid: u32,
    pub comm: String,
    pub norm_cpu: f32,
    pub norm_io: f32,
    pub norm_mem: f32,
    pub norm_threads: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkloadClassificationResponse {
    pub category: String,
    pub logits: Vec<f32>,
}
