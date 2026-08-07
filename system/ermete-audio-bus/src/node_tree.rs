use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    AudioSink,       // Hardware or Virtual Output (Speakers, Headphones)
    AudioSource,     // Hardware or Virtual Input (Microphone, Line-In)
    AppOutputStream, // Application Playback Stream (e.g., Spotify, Chrome)
    AppInputStream,  // Application Capture Stream (e.g., Discord, OBS)
    VirtualSink,     // Swarm-created Virtual Output
    VirtualSource,   // Swarm-created Virtual Input
    DspFilter,       // Echo cancellation, noise reduction, spatializer
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPort {
    pub id: u32,
    pub node_id: u32,
    pub name: String,
    pub direction: PortDirection,
    pub channel_type: String, // e.g., "FL", "FR", "MONO", "AUX0"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioNode {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub media_class: String,
    pub node_type: NodeType,
    pub volume: f32, // 0.0 to 1.0 (or >1.0 for gain)
    pub muted: bool,
    pub is_default: bool,
    pub priority: u32,
    pub ports: Vec<AudioPort>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkState {
    Active,
    Paused,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioLink {
    pub id: u32,
    pub output_node_id: u32,
    pub output_port_id: u32,
    pub input_node_id: u32,
    pub input_port_id: u32,
    pub state: LinkState,
    pub created_by_swarm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTreeState {
    pub sinks: Vec<AudioNode>,
    pub sources: Vec<AudioNode>,
    pub streams: Vec<AudioNode>,
    pub links: Vec<AudioLink>,
    pub default_sink_id: Option<u32>,
    pub default_source_id: Option<u32>,
}

#[derive(Clone)]
pub struct NodeTree {
    state: Arc<RwLock<NodeTreeState>>,
}

impl NodeTree {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(NodeTreeState {
                sinks: Vec::new(),
                sources: Vec::new(),
                streams: Vec::new(),
                links: Vec::new(),
                default_sink_id: None,
                default_source_id: None,
            })),
        }
    }

    pub async fn get_snapshot(&self) -> NodeTreeState {
        self.state.read().await.clone()
    }

    pub async fn add_or_update_node(&self, node: AudioNode) {
        let mut guard = self.state.write().await;
        match node.node_type {
            NodeType::AudioSink | NodeType::VirtualSink => {
                if let Some(pos) = guard.sinks.iter().position(|n| n.id == node.id) {
                    guard.sinks[pos] = node;
                } else {
                    if guard.default_sink_id.is_none() {
                        guard.default_sink_id = Some(node.id);
                    }
                    guard.sinks.push(node);
                }
            }
            NodeType::AudioSource | NodeType::VirtualSource => {
                if let Some(pos) = guard.sources.iter().position(|n| n.id == node.id) {
                    guard.sources[pos] = node;
                } else {
                    if guard.default_source_id.is_none() {
                        guard.default_source_id = Some(node.id);
                    }
                    guard.sources.push(node);
                }
            }
            _ => {
                if let Some(pos) = guard.streams.iter().position(|n| n.id == node.id) {
                    guard.streams[pos] = node;
                } else {
                    guard.streams.push(node);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub async fn remove_node(&self, node_id: u32) {
        let mut guard = self.state.write().await;
        guard.sinks.retain(|n| n.id != node_id);
        guard.sources.retain(|n| n.id != node_id);
        guard.streams.retain(|n| n.id != node_id);
        guard.links.retain(|l| l.output_node_id != node_id && l.input_node_id != node_id);

        if guard.default_sink_id == Some(node_id) {
            guard.default_sink_id = guard.sinks.first().map(|n| n.id);
        }
        if guard.default_source_id == Some(node_id) {
            guard.default_source_id = guard.sources.first().map(|n| n.id);
        }
    }

    pub async fn set_default_sink(&self, sink_id: u32) -> Result<(), String> {
        let mut guard = self.state.write().await;
        if !guard.sinks.iter().any(|n| n.id == sink_id) {
            return Err(format!("Sink node with ID {} not found", sink_id));
        }
        for sink in guard.sinks.iter_mut() {
            sink.is_default = sink.id == sink_id;
        }
        guard.default_sink_id = Some(sink_id);
        Ok(())
    }

    pub async fn set_default_source(&self, source_id: u32) -> Result<(), String> {
        let mut guard = self.state.write().await;
        if !guard.sources.iter().any(|n| n.id == source_id) {
            return Err(format!("Source node with ID {} not found", source_id));
        }
        for source in guard.sources.iter_mut() {
            source.is_default = source.id == source_id;
        }
        guard.default_source_id = Some(source_id);
        Ok(())
    }

    pub async fn set_volume(&self, node_id: u32, volume: f32) -> Result<(), String> {
        let mut guard = self.state.write().await;
        let clamped = volume.clamp(0.0, 2.0); // Allow up to 200% amplification
        let mut found = false;

        let NodeTreeState {
            ref mut sinks,
            ref mut sources,
            ref mut streams,
            ..
        } = *guard;

        for node in sinks.iter_mut().chain(sources.iter_mut()).chain(streams.iter_mut()) {
            if node.id == node_id {
                node.volume = clamped;
                found = true;
                break;
            }
        }

        if found {
            Ok(())
        } else {
            Err(format!("Node ID {} not found", node_id))
        }
    }

    pub async fn set_mute(&self, node_id: u32, mute: bool) -> Result<(), String> {
        let mut guard = self.state.write().await;
        let mut found = false;

        let NodeTreeState {
            ref mut sinks,
            ref mut sources,
            ref mut streams,
            ..
        } = *guard;

        for node in sinks.iter_mut().chain(sources.iter_mut()).chain(streams.iter_mut()) {
            if node.id == node_id {
                node.muted = mute;
                found = true;
                break;
            }
        }

        if found {
            Ok(())
        } else {
            Err(format!("Node ID {} not found", node_id))
        }
    }

    pub async fn add_link(&self, link: AudioLink) {
        let mut guard = self.state.write().await;
        if !guard.links.iter().any(|l| l.id == link.id) {
            guard.links.push(link);
        }
    }

    pub async fn remove_link(&self, link_id: u32) -> Result<(), String> {
        let mut guard = self.state.write().await;
        let initial_len = guard.links.len();
        guard.links.retain(|l| l.id != link_id);
        if guard.links.len() < initial_len {
            Ok(())
        } else {
            Err(format!("Link ID {} not found", link_id))
        }
    }
}
