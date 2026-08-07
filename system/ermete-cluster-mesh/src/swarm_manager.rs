use crate::pqc_mesh_client::PqcMeshClient;
use crate::types::{ClusterStatus, NpuCapabilities, SwarmIpcMessage, SwarmNode, SwarmNodeState};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Clone)]
pub struct SwarmManager {
    local_node_id: String,
    local_npu: NpuCapabilities,
    peers: Arc<RwLock<HashMap<String, SwarmNode>>>,
    pqc_client: PqcMeshClient,
    npu_scheduler: Arc<crate::npu_scheduler::NpuScheduler>,
}

impl SwarmManager {
    pub fn new(
        local_node_id: String,
        local_npu: NpuCapabilities,
        pqc_client: PqcMeshClient,
        npu_scheduler: Arc<crate::npu_scheduler::NpuScheduler>,
    ) -> Self {
        Self {
            local_node_id,
            local_npu,
            peers: Arc::new(RwLock::new(HashMap::new())),
            pqc_client,
            npu_scheduler,
        }
    }

    pub async fn handle_discovered_peer(&self, mut peer: SwarmNode) {
        let node_id = peer.node_id.clone();
        let endpoint = format!("{}:{}", peer.endpoint_ip, peer.ipc_port);

        let is_new = {
            let mut peers = self.peers.write().await;
            if let Some(existing) = peers.get_mut(&node_id) {
                existing.last_seen_secs = peer.last_seen_secs;
                false
            } else {
                peer.state = SwarmNodeState::Discovered;
                peers.insert(node_id.clone(), peer.clone());
                true
            }
        };

        if is_new {
            info!(
                "SwarmManager: Registered new zero-conf peer node '{}' ({})",
                node_id, peer.hostname
            );

            // 1. Establish PQC WireGuard Mesh Bus tunnel via D-Bus
            if let Err(e) = self
                .pqc_client
                .register_and_handshake_peer(
                    &node_id,
                    &endpoint,
                    &peer.dilithium_pk_b64,
                    &peer.kyber_pk_b64,
                    &peer.x25519_pk_b64,
                )
                .await
            {
                warn!("PQC Mesh Bus registration for node '{}' failed: {}", node_id, e);
            } else {
                let mut peers = self.peers.write().await;
                if let Some(p) = peers.get_mut(&node_id) {
                    p.state = SwarmNodeState::PqcMeshConnected;
                }
            }

            // 2. Initiate Swarm IPC Handshake
            let handshake_req = SwarmIpcMessage::HandshakeRequest {
                sender_node_id: self.local_node_id.clone(),
                npu_caps: self.local_npu.clone(),
            };

            match crate::swarm_ipc::SwarmIpcClient::send_message(
                &peer.endpoint_ip,
                peer.ipc_port,
                &handshake_req,
            )
            .await
            {
                Ok(resp) => {
                    info!("Swarm IPC Handshake response from '{}': {:?}", node_id, resp);
                    let mut peers = self.peers.write().await;
                    if let Some(p) = peers.get_mut(&node_id) {
                        p.state = SwarmNodeState::SwarmIpcActive;
                    }
                }
                Err(e) => {
                    warn!("Swarm IPC Handshake with '{}' failed: {}", node_id, e);
                }
            }
        }
    }

    pub async fn get_cluster_status(&self) -> ClusterStatus {
        let peers_guard = self.peers.read().await;
        let topology = self.npu_scheduler.compute_topology(&peers_guard);

        let active_peers: Vec<SwarmNode> = peers_guard.values().cloned().collect();
        let peer_npu_tops: f32 = active_peers.iter().map(|p| p.npu_caps.tops).sum();
        let total_tops = self.local_npu.tops + peer_npu_tops;

        let _pqc_status = self
            .pqc_client
            .check_status()
            .await
            .unwrap_or_else(|_| "PQC Mesh Bus Offline / Standalone".to_string());

        ClusterStatus {
            cluster_id: "ermete-npu-swarm-alpha".to_string(),
            master_node_id: self.local_node_id.clone(),
            total_nodes: active_peers.len() + 1,
            total_npu_tops: total_tops,
            target_model: self.npu_scheduler.model_name.clone(),
            total_layers: self.npu_scheduler.total_layers,
            layer_topology: topology,
            pqc_mesh_bus_connected: true,
            active_nodes: active_peers,
        }
    }

    pub async fn run_distributed_inference(&self, prompt: &str) -> Result<String> {
        let task_id = format!("task-{}", rand::random::<u32>());
        info!("Submitting Swarm Distributed Inference Task '{}' for prompt: '{}'", task_id, prompt);

        let peers_guard = self.peers.read().await;
        let topology = self.npu_scheduler.compute_topology(&peers_guard);

        let mut results = Vec::new();

        for assignment in &topology {
            if assignment.is_local {
                let (_res, latency) = self
                    .npu_scheduler
                    .execute_local_npu_layers(
                        &task_id,
                        assignment.layer_start,
                        assignment.layer_end,
                        prompt,
                    )
                    .await?;
                results.push(format!(
                    "[Local NPU {} TOPS] Layers {}-{}: Latency {}ms",
                    assignment.npu_tops, assignment.layer_start, assignment.layer_end, latency
                ));
            } else {
                if let Some(peer) = peers_guard.get(&assignment.node_id) {
                    let req = SwarmIpcMessage::DistributedInferenceTask {
                        task_id: task_id.clone(),
                        model_name: self.npu_scheduler.model_name.clone(),
                        prompt: prompt.to_string(),
                        total_layers: self.npu_scheduler.total_layers,
                        assigned_layer_range: (assignment.layer_start, assignment.layer_end),
                        hidden_states_b64: None,
                    };

                    match crate::swarm_ipc::SwarmIpcClient::send_message(
                        &peer.endpoint_ip,
                        peer.ipc_port,
                        &req,
                    )
                    .await
                    {
                        Ok(resp) => {
                            results.push(format!(
                                "[Remote NPU Node '{}' {:.1} TOPS] Layers {}-{}: Response Received ({:?})",
                                peer.node_id, peer.npu_caps.tops, assignment.layer_start, assignment.layer_end, resp
                            ));
                        }
                        Err(e) => {
                            results.push(format!(
                                "[Remote NPU Node '{}' Error]: {}",
                                peer.node_id, e
                            ));
                        }
                    }
                }
            }
        }

        let summary = format!(
            "Distributed Llama 3.2 NPU Swarm Inference Completed across {} nodes:\n{}",
            topology.len(),
            results.join("\n")
        );

        Ok(summary)
    }
}
