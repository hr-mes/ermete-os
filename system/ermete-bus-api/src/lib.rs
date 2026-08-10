use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zbus::proxy;

pub mod shm_ring;
pub use shm_ring::*;

pub mod pqc;
pub mod socket;


/// Common telemetry payload collected from Ring-0 eBPF probes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KernelTelemetry {
    pub timestamp_secs: u64,
    pub syscall_frequency_hz: u64,
    pub memory_pressure_mb: u64,
    pub network_passed_packets: u64,
    pub network_dropped_packets: u64,
    pub land_attacks_detected: u64,
    pub tcp_scans_detected: u64,
    pub blocklist_drops: u64,
    pub unauthorized_port_drops: u64,
}

/// Mesh peer metadata structure shared across services.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MeshPeerInfo {
    pub node_id: String,
    pub endpoint: Option<String>,
    pub virtual_ip: String,
    pub dilithium_pk_b64: String,
    pub kyber_pk_b64: String,
    pub x25519_pk_b64: String,
    pub state: String,
    pub last_handshake: u64,
    pub packets_rx: u64,
    pub packets_tx: u64,
    pub latency_ms: u32,
    pub zero_trust_verified: bool,
}

/// Post-Quantum Node Identity payload structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeIdentityPayload {
    pub node_id: String,
    pub x25519_public_b64: String,
    pub kyber_public_b64: String,
    pub dilithium_public_b64: String,
    pub pqc_level: String,
}

/// AI Decision Payload returned by the NPU inference engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDecisionPayload {
    pub anomaly_detected: bool,
    pub risk_score: f32,
    pub recommended_actions: Vec<String>,
    pub sysctl_mitigations: Vec<(String, String)>,
    pub block_ips: Vec<String>,
    pub zero_trust_enforce: bool,
}

/// High-level status payload for the PQC Mesh Bus.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MeshBusStatusPayload {
    pub node_id: String,
    pub status: String,
    pub active_peers: usize,
    pub pqc_algorithm: String,
    pub zero_trust_enabled: bool,
}

/// Dynamic trait for interacting with the PQC Mesh Bus.
#[async_trait]
pub trait MeshBusClientTrait: Send + Sync {
    async fn get_status(&self) -> anyhow::Result<MeshBusStatusPayload>;
    async fn get_peers(&self) -> anyhow::Result<Vec<MeshPeerInfo>>;
    async fn get_node_identity(&self) -> anyhow::Result<NodeIdentityPayload>;
    async fn initiate_handshake(&self, node_id: &str, endpoint: &str) -> anyhow::Result<String>;
}

/// Dynamic trait for collecting Ring-0 telemetry.
#[async_trait]
pub trait KernelTelemetryProvider: Send + Sync {
    async fn collect_telemetry(&mut self) -> KernelTelemetry;
}

/// Dynamic trait for applying autonomic policies and mitigations.
#[async_trait]
pub trait AutonomicPolicyEnforcer: Send + Sync {
    async fn apply_mitigations(&self, decision: &AiDecisionPayload) -> anyhow::Result<()>;
}

/// Shared DBus interface proxy definition for org.ermete.MeshBus
#[proxy(
    interface = "org.ermete.MeshBus",
    default_service = "org.ermete.MeshBus",
    default_path = "/org/ermete/MeshBus"
)]
pub trait MeshBusInterface {
    async fn status(&self) -> zbus::Result<String>;
    async fn get_pqc_capabilities(&self) -> zbus::Result<String>;
    async fn get_peers(&self) -> zbus::Result<String>;
    async fn add_peer(
        &self,
        node_id: String,
        endpoint: String,
        dilithium_pk_b64: String,
        kyber_pk_b64: String,
        x25519_pk_b64: String,
    ) -> zbus::Result<String>;
    async fn remove_peer(&self, node_id: String) -> zbus::Result<String>;
    async fn initiate_handshake(&self, node_id: String, endpoint: String) -> zbus::Result<String>;
    async fn get_node_identity(&self) -> zbus::Result<String>;
    async fn rotate_keys(&self) -> zbus::Result<String>;
}
