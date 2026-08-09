use crate::node_tree::NodeTree;
use crate::routing::RoutingEngine;
use std::sync::Arc;

/// Micro-service responsible for node audio controls (volume, mute, default sink/source).
#[derive(Clone)]
pub struct AudioDeviceController {
    node_tree: NodeTree,
    routing_engine: Arc<RoutingEngine>,
}

impl AudioDeviceController {
    pub fn new(node_tree: NodeTree, routing_engine: Arc<RoutingEngine>) -> Self {
        Self {
            node_tree,
            routing_engine,
        }
    }

    pub async fn set_default_sink(&self, node_id: u32) -> String {
        match self.node_tree.set_default_sink(node_id).await {
            Ok(_) => {
                self.routing_engine.reevaluate_routing().await;
                format!("Successfully set default sink to node {}", node_id)
            }
            Err(e) => format!("Error setting default sink: {}", e),
        }
    }

    pub async fn set_default_source(&self, node_id: u32) -> String {
        match self.node_tree.set_default_source(node_id).await {
            Ok(_) => format!("Successfully set default source to node {}", node_id),
            Err(e) => format!("Error setting default source: {}", e),
        }
    }

    pub async fn set_node_volume(&self, node_id: u32, volume: f32) -> String {
        match self.node_tree.set_volume(node_id, volume).await {
            Ok(_) => format!("Set volume for node {} to {:.2}", node_id, volume),
            Err(e) => format!("Error setting volume: {}", e),
        }
    }

    pub async fn set_node_mute(&self, node_id: u32, mute: bool) -> String {
        match self.node_tree.set_mute(node_id, mute).await {
            Ok(_) => format!("Set mute status for node {} to {}", node_id, mute),
            Err(e) => format!("Error setting mute status: {}", e),
        }
    }
}
