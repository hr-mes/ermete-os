use crate::node_tree::{AudioLink, AudioNode, LinkState, NodeTree, NodeType};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmRoutingRule {
    pub rule_id: String,
    pub match_app_name: Option<String>,
    pub match_media_category: Option<String>, // e.g., "ai-voice", "communication", "music"
    pub target_sink_name: Option<String>,
    pub priority: u32,
    pub auto_mute_other_streams: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmRoutingPolicy {
    pub policy_name: String,
    pub auto_connect_new_streams: bool,
    pub prefer_bluetooth_on_connect: bool,
    pub ai_voice_ducking_db: f32, // Lower non-AI volume when AI voice speaks
    pub rules: Vec<SwarmRoutingRule>,
}

impl Default for SwarmRoutingPolicy {
    fn default() -> Self {
        Self {
            policy_name: "Ermete OS Default Swarm Policy".to_string(),
            auto_connect_new_streams: true,
            prefer_bluetooth_on_connect: true,
            ai_voice_ducking_db: -6.0,
            rules: vec![
                SwarmRoutingRule {
                    rule_id: "rule-ai-assistant".to_string(),
                    match_app_name: Some("ermete-ai-daemon".to_string()),
                    match_media_category: Some("ai-voice".to_string()),
                    target_sink_name: None,
                    priority: 100,
                    auto_mute_other_streams: false,
                },
                SwarmRoutingRule {
                    rule_id: "rule-system-alerts".to_string(),
                    match_app_name: Some("ermete-shell".to_string()),
                    match_media_category: Some("notification".to_string()),
                    target_sink_name: None,
                    priority: 90,
                    auto_mute_other_streams: false,
                },
            ],
        }
    }
}

pub struct RoutingEngine {
    node_tree: NodeTree,
    policy: Arc<RwLock<SwarmRoutingPolicy>>,
    next_link_id: Arc<RwLock<u32>>,
}

impl RoutingEngine {
    pub fn new(node_tree: NodeTree) -> Self {
        Self {
            node_tree,
            policy: Arc::new(RwLock::new(SwarmRoutingPolicy::default())),
            next_link_id: Arc::new(RwLock::new(1000)),
        }
    }

    pub async fn get_policy(&self) -> SwarmRoutingPolicy {
        self.policy.read().await.clone()
    }

    pub async fn set_policy(&self, new_policy: SwarmRoutingPolicy) {
        info!("Updating Swarm Audio Routing Policy: {}", new_policy.policy_name);
        *self.policy.write().await = new_policy;
        self.reevaluate_routing().await;
    }

    pub async fn reevaluate_routing(&self) {
        let tree_snapshot = self.node_tree.get_snapshot().await;
        let policy_guard = self.policy.read().await;

        if !policy_guard.auto_connect_new_streams {
            return;
        }

        // Identify active playback streams without a link
        for stream in &tree_snapshot.streams {
            if stream.node_type != NodeType::AppOutputStream {
                continue;
            }

            let is_linked = tree_snapshot
                .links
                .iter()
                .any(|link| link.output_node_id == stream.id);

            if !is_linked {
                // Route stream to default sink or rule-matching sink
                if let Some(target_sink) = self.find_best_sink_for_stream(stream, &tree_snapshot.sinks, &policy_guard).await {
                    self.connect_stream_to_sink(stream, &target_sink).await;
                }
            }
        }
    }

    async fn find_best_sink_for_stream<'a>(
        &self,
        stream: &AudioNode,
        sinks: &'a [AudioNode],
        policy: &SwarmRoutingPolicy,
    ) -> Option<AudioNode> {
        if sinks.is_empty() {
            return None;
        }

        // Check matching rule
        for rule in &policy.rules {
            if let Some(app_name) = &rule.match_app_name {
                if stream.name.contains(app_name) || stream.description.contains(app_name) {
                    if let Some(target_name) = &rule.target_sink_name {
                        if let Some(sink) = sinks.iter().find(|s| s.name.contains(target_name)) {
                            return Some(sink.clone());
                        }
                    }
                }
            }
        }

        // If Bluetooth preference enabled and Bluetooth sink present, pick it
        if policy.prefer_bluetooth_on_connect {
            if let Some(bt_sink) = sinks.iter().find(|s| s.name.contains("bluez") || s.description.to_lowercase().contains("bluetooth")) {
                return Some(bt_sink.clone());
            }
        }

        // Default to marked default sink or first sink
        sinks.iter().find(|s| s.is_default).cloned().or_else(|| sinks.first().cloned())
    }

    pub async fn connect_stream_to_sink(&self, stream: &AudioNode, sink: &AudioNode) {
        let mut link_id_guard = self.next_link_id.write().await;
        *link_id_guard += 1;
        let link_id = *link_id_guard;

        let output_port = stream.ports.first().map(|p| p.id).unwrap_or(0);
        let input_port = sink.ports.first().map(|p| p.id).unwrap_or(0);

        info!(
            "Swarm Routing Engine: Connecting stream '{}' (node {}) -> sink '{}' (node {})",
            stream.name, stream.id, sink.name, sink.id
        );

        let link = AudioLink {
            id: link_id,
            output_node_id: stream.id,
            output_port_id: output_port,
            input_node_id: sink.id,
            input_port_id: input_port,
            state: LinkState::Active,
            created_by_swarm: true,
        };

        self.node_tree.add_link(link).await;
    }

    pub async fn manual_create_link(&self, output_node_id: u32, input_node_id: u32) -> Result<u32, String> {
        let tree_snapshot = self.node_tree.get_snapshot().await;

        let out_node = tree_snapshot
            .sinks
            .iter()
            .chain(&tree_snapshot.sources)
            .chain(&tree_snapshot.streams)
            .find(|n| n.id == output_node_id)
            .ok_or_else(|| format!("Output node ID {} not found", output_node_id))?;

        let in_node = tree_snapshot
            .sinks
            .iter()
            .chain(&tree_snapshot.sources)
            .chain(&tree_snapshot.streams)
            .find(|n| n.id == input_node_id)
            .ok_or_else(|| format!("Input node ID {} not found", input_node_id))?;

        let mut link_id_guard = self.next_link_id.write().await;
        *link_id_guard += 1;
        let link_id = *link_id_guard;

        let out_port = out_node.ports.first().map(|p| p.id).unwrap_or(0);
        let in_port = in_node.ports.first().map(|p| p.id).unwrap_or(0);

        let link = AudioLink {
            id: link_id,
            output_node_id: out_node.id,
            output_port_id: out_port,
            input_node_id: in_node.id,
            input_port_id: in_port,
            state: LinkState::Active,
            created_by_swarm: true,
        };

        self.node_tree.add_link(link).await;
        info!("Manually established link {} (node {} -> node {})", link_id, output_node_id, input_node_id);
        Ok(link_id)
    }
}
