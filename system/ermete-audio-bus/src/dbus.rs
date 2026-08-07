use crate::node_tree::NodeTree;
use crate::pipewire_manager::PipewireManager;
use crate::routing::{RoutingEngine, SwarmRoutingPolicy};
use std::sync::Arc;
use zbus::interface;

pub struct AudioBusInterface {
    node_tree: NodeTree,
    routing_engine: Arc<RoutingEngine>,
    pw_manager: Arc<PipewireManager>,
}

impl AudioBusInterface {
    pub fn new(
        node_tree: NodeTree,
        routing_engine: Arc<RoutingEngine>,
        pw_manager: Arc<PipewireManager>,
    ) -> Self {
        Self {
            node_tree,
            routing_engine,
            pw_manager,
        }
    }
}

#[interface(name = "org.ermete.AudioBus")]
impl AudioBusInterface {
    async fn status(&self) -> String {
        serde_json::json!({
            "service": "ermete-audio-bus",
            "version": "1.0.0",
            "status": "ACTIVE",
            "mode": "Native Rust PipeWire Session Manager",
            "wireplumber_replacement": true,
            "swarm_autonomous_routing": true
        })
        .to_string()
    }

    async fn get_audio_node_tree(&self) -> String {
        let snapshot = self.node_tree.get_snapshot().await;
        serde_json::to_string_pretty(&snapshot).unwrap_or_else(|_| "{}".to_string())
    }

    async fn get_sinks(&self) -> String {
        let snapshot = self.node_tree.get_snapshot().await;
        serde_json::to_string_pretty(&snapshot.sinks).unwrap_or_else(|_| "[]".to_string())
    }

    async fn get_sources(&self) -> String {
        let snapshot = self.node_tree.get_snapshot().await;
        serde_json::to_string_pretty(&snapshot.sources).unwrap_or_else(|_| "[]".to_string())
    }

    async fn get_links(&self) -> String {
        let snapshot = self.node_tree.get_snapshot().await;
        serde_json::to_string_pretty(&snapshot.links).unwrap_or_else(|_| "[]".to_string())
    }

    async fn set_default_sink(&self, node_id: u32) -> String {
        match self.node_tree.set_default_sink(node_id).await {
            Ok(_) => {
                self.routing_engine.reevaluate_routing().await;
                format!("Successfully set default sink to node {}", node_id)
            }
            Err(e) => format!("Error setting default sink: {}", e),
        }
    }

    async fn set_default_source(&self, node_id: u32) -> String {
        match self.node_tree.set_default_source(node_id).await {
            Ok(_) => format!("Successfully set default source to node {}", node_id),
            Err(e) => format!("Error setting default source: {}", e),
        }
    }

    async fn set_node_volume(&self, node_id: u32, volume: f32) -> String {
        match self.node_tree.set_volume(node_id, volume).await {
            Ok(_) => format!("Set volume for node {} to {:.2}", node_id, volume),
            Err(e) => format!("Error setting volume: {}", e),
        }
    }

    async fn set_node_mute(&self, node_id: u32, mute: bool) -> String {
        match self.node_tree.set_mute(node_id, mute).await {
            Ok(_) => format!("Set mute status for node {} to {}", node_id, mute),
            Err(e) => format!("Error setting mute status: {}", e),
        }
    }

    async fn create_link(&self, output_node_id: u32, input_node_id: u32) -> String {
        match self.routing_engine.manual_create_link(output_node_id, input_node_id).await {
            Ok(link_id) => format!("Created audio link {} between node {} and node {}", link_id, output_node_id, input_node_id),
            Err(e) => format!("Error creating audio link: {}", e),
        }
    }

    async fn remove_link(&self, link_id: u32) -> String {
        match self.node_tree.remove_link(link_id).await {
            Ok(_) => format!("Successfully removed audio link {}", link_id),
            Err(e) => format!("Error removing link: {}", e),
        }
    }

    async fn create_virtual_sink(&self, name: String, channels: u32) -> String {
        match self.pw_manager.create_virtual_sink(name, channels).await {
            Ok(id) => format!("Created virtual sink with ID {}", id),
            Err(e) => format!("Error creating virtual sink: {}", e),
        }
    }

    async fn get_swarm_routing_policy(&self) -> String {
        let policy = self.routing_engine.get_policy().await;
        serde_json::to_string_pretty(&policy).unwrap_or_else(|_| "{}".to_string())
    }

    async fn set_swarm_routing_policy(&self, policy_json: String) -> String {
        match serde_json::from_str::<SwarmRoutingPolicy>(&policy_json) {
            Ok(policy) => {
                self.routing_engine.set_policy(policy).await;
                "Swarm Routing Policy successfully applied".to_string()
            }
            Err(e) => format!("Failed to parse routing policy JSON: {}", e),
        }
    }
}
