use crate::node_tree::NodeTree;
use crate::routing::{RoutingEngine, SwarmRoutingPolicy};
use std::sync::Arc;

/// Micro-service responsible for dynamic audio links and swarm routing policies.
#[derive(Clone)]
pub struct AudioRoutingService {
    node_tree: NodeTree,
    routing_engine: Arc<RoutingEngine>,
}

impl AudioRoutingService {
    pub fn new(node_tree: NodeTree, routing_engine: Arc<RoutingEngine>) -> Self {
        Self {
            node_tree,
            routing_engine,
        }
    }

    pub async fn create_link(&self, output_node_id: u32, input_node_id: u32) -> String {
        match self.routing_engine.manual_create_link(output_node_id, input_node_id).await {
            Ok(link_id) => format!("Created audio link {} between node {} and node {}", link_id, output_node_id, input_node_id),
            Err(e) => format!("Error creating audio link: {}", e),
        }
    }

    pub async fn remove_link(&self, link_id: u32) -> String {
        match self.node_tree.remove_link(link_id).await {
            Ok(_) => format!("Successfully removed audio link {}", link_id),
            Err(e) => format!("Error removing link: {}", e),
        }
    }

    pub async fn get_swarm_routing_policy(&self) -> String {
        let policy = self.routing_engine.get_policy().await;
        serde_json::to_string_pretty(&policy).unwrap_or_else(|_| "{}".to_string())
    }

    pub async fn set_swarm_routing_policy(&self, policy_json: String) -> String {
        match serde_json::from_str::<SwarmRoutingPolicy>(&policy_json) {
            Ok(policy) => {
                self.routing_engine.set_policy(policy).await;
                "Swarm Routing Policy successfully applied".to_string()
            }
            Err(e) => format!("Failed to parse routing policy JSON: {}", e),
        }
    }
}
