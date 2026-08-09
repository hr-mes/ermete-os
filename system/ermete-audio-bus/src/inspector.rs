use crate::node_tree::NodeTree;
use serde_json::json;

/// Micro-service responsible for inspecting audio node graphs, sinks, sources, and status.
#[derive(Clone)]
pub struct AudioNodeInspector {
    node_tree: NodeTree,
}

impl AudioNodeInspector {
    pub fn new(node_tree: NodeTree) -> Self {
        Self { node_tree }
    }

    pub async fn get_status(&self) -> String {
        json!({
            "service": "ermete-audio-bus",
            "version": "1.0.0",
            "status": "ACTIVE",
            "mode": "Native Rust PipeWire Session Manager",
            "wireplumber_replacement": true,
            "swarm_autonomous_routing": true
        })
        .to_string()
    }

    pub async fn get_node_tree(&self) -> String {
        let snapshot = self.node_tree.get_snapshot().await;
        serde_json::to_string_pretty(&snapshot).unwrap_or_else(|_| "{}".to_string())
    }

    pub async fn get_sinks(&self) -> String {
        let snapshot = self.node_tree.get_snapshot().await;
        serde_json::to_string_pretty(&snapshot.sinks).unwrap_or_else(|_| "[]".to_string())
    }

    pub async fn get_sources(&self) -> String {
        let snapshot = self.node_tree.get_snapshot().await;
        serde_json::to_string_pretty(&snapshot.sources).unwrap_or_else(|_| "[]".to_string())
    }

    pub async fn get_links(&self) -> String {
        let snapshot = self.node_tree.get_snapshot().await;
        serde_json::to_string_pretty(&snapshot.links).unwrap_or_else(|_| "[]".to_string())
    }
}
