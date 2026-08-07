use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpuCapabilities {
    pub device_name: String,
    pub backend: String, // "Vulkan-Tensor", "Qualcomm-Hexagon-NPU", "Intel-NPU", "eBPF-NPU-Offload"
    pub tops: f32,       // e.g. 45.0 TOPS
    pub total_vram_mb: u32,
    pub free_vram_mb: u32,
    pub supports_llama3_2: bool,
    pub cpu_impact_percentage: f32, // Target: 0.0%
}

impl Default for NpuCapabilities {
    fn default() -> Self {
        Self {
            device_name: "Ermete NPU Tensor Accelerator".to_string(),
            backend: "Vulkan-NPU-Direct".to_string(),
            tops: 45.0,
            total_vram_mb: 16384,
            free_vram_mb: 12288,
            supports_llama3_2: true,
            cpu_impact_percentage: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmBeacon {
    pub node_id: String,
    pub hostname: String,
    pub endpoint_ip: String,
    pub discovery_port: u16,
    pub ipc_port: u16,
    pub dilithium_pk_b64: String,
    pub kyber_pk_b64: String,
    pub x25519_pk_b64: String,
    pub npu_caps: NpuCapabilities,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwarmNodeState {
    Discovered,
    PqcMeshConnected,
    SwarmIpcActive,
    OffloadingActive,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmNode {
    pub node_id: String,
    pub hostname: String,
    pub endpoint_ip: String,
    pub ipc_port: u16,
    pub virtual_ip: Option<String>,
    pub dilithium_pk_b64: String,
    pub kyber_pk_b64: String,
    pub x25519_pk_b64: String,
    pub npu_caps: NpuCapabilities,
    pub state: SwarmNodeState,
    pub assigned_layer_range: (u32, u32), // e.g. (0, 14) or (15, 27)
    pub last_seen_secs: u64,
    pub pqc_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwarmIpcMessage {
    HandshakeRequest {
        sender_node_id: String,
        npu_caps: NpuCapabilities,
    },
    HandshakeResponse {
        sender_node_id: String,
        accepted: bool,
        cluster_master: String,
    },
    DistributedInferenceTask {
        task_id: String,
        model_name: String,
        prompt: String,
        total_layers: u32,
        assigned_layer_range: (u32, u32),
        hidden_states_b64: Option<String>,
    },
    DistributedInferenceResult {
        task_id: String,
        node_id: String,
        layer_range_executed: (u32, u32),
        next_hidden_states_b64: Option<String>,
        generated_text: Option<String>,
        logits_b64: Option<String>,
        latency_ms: u64,
        npu_tops_utilized: f32,
    },
    Heartbeat {
        node_id: String,
        npu_load_percent: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLayerAssignment {
    pub node_id: String,
    pub hostname: String,
    pub npu_tops: f32,
    pub layer_start: u32,
    pub layer_end: u32,
    pub is_local: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub cluster_id: String,
    pub master_node_id: String,
    pub total_nodes: usize,
    pub total_npu_tops: f32,
    pub target_model: String,
    pub total_layers: u32,
    pub layer_topology: Vec<NodeLayerAssignment>,
    pub pqc_mesh_bus_connected: bool,
    pub active_nodes: Vec<SwarmNode>,
}
