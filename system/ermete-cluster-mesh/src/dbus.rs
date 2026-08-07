use crate::swarm_manager::SwarmManager;
use std::sync::Arc;
use zbus::interface;

pub struct ClusterMeshInterface {
    swarm_manager: Arc<SwarmManager>,
}

impl ClusterMeshInterface {
    pub fn new(swarm_manager: Arc<SwarmManager>) -> Self {
        Self { swarm_manager }
    }
}

#[interface(name = "org.ermete.ClusterMesh")]
impl ClusterMeshInterface {
    async fn status(&self) -> String {
        let status = self.swarm_manager.get_cluster_status().await;
        format!(
            "Ermete OS Cluster Mesh ACTIVE [Nodes: {}, Total NPU TOPS: {:.1}, Model: {}]",
            status.total_nodes, status.total_npu_tops, status.target_model
        )
    }

    async fn get_cluster_status(&self) -> String {
        let status = self.swarm_manager.get_cluster_status().await;
        serde_json::to_string_pretty(&status).unwrap_or_else(|_| "{}".to_string())
    }

    async fn get_discovered_nodes(&self) -> String {
        let status = self.swarm_manager.get_cluster_status().await;
        serde_json::to_string_pretty(&status.active_nodes).unwrap_or_else(|_| "[]".to_string())
    }

    async fn get_npu_topology(&self) -> String {
        let status = self.swarm_manager.get_cluster_status().await;
        serde_json::to_string_pretty(&status.layer_topology).unwrap_or_else(|_| "[]".to_string())
    }

    async fn submit_distributed_inference(&self, prompt: String) -> String {
        match self.swarm_manager.run_distributed_inference(&prompt).await {
            Ok(res) => res,
            Err(e) => format!("Distributed NPU Inference Error: {}", e),
        }
    }
}
