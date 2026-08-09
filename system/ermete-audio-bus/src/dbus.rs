use crate::controller::AudioDeviceController;
use crate::inspector::AudioNodeInspector;
use crate::node_tree::NodeTree;
use crate::pipewire_manager::PipewireManager;
use crate::routing::RoutingEngine;
use crate::routing_service::AudioRoutingService;
use crate::virtual_device::AudioVirtualDeviceService;
use std::sync::Arc;
use zbus::interface;

pub struct AudioBusInterface {
    pub inspector: AudioNodeInspector,
    pub controller: AudioDeviceController,
    pub routing: AudioRoutingService,
    pub virtual_devices: AudioVirtualDeviceService,
}

impl AudioBusInterface {
    pub fn new(
        node_tree: NodeTree,
        routing_engine: Arc<RoutingEngine>,
        pw_manager: Arc<PipewireManager>,
    ) -> Self {
        Self {
            inspector: AudioNodeInspector::new(node_tree.clone()),
            controller: AudioDeviceController::new(node_tree.clone(), routing_engine.clone()),
            routing: AudioRoutingService::new(node_tree, routing_engine),
            virtual_devices: AudioVirtualDeviceService::new(pw_manager),
        }
    }
}

#[interface(name = "org.ermete.AudioBus")]
impl AudioBusInterface {
    async fn status(&self) -> String {
        self.inspector.get_status().await
    }

    async fn get_audio_node_tree(&self) -> String {
        self.inspector.get_node_tree().await
    }

    async fn get_sinks(&self) -> String {
        self.inspector.get_sinks().await
    }

    async fn get_sources(&self) -> String {
        self.inspector.get_sources().await
    }

    async fn get_links(&self) -> String {
        self.inspector.get_links().await
    }

    async fn set_default_sink(&self, node_id: u32) -> String {
        self.controller.set_default_sink(node_id).await
    }

    async fn set_default_source(&self, node_id: u32) -> String {
        self.controller.set_default_source(node_id).await
    }

    async fn set_node_volume(&self, node_id: u32, volume: f32) -> String {
        self.controller.set_node_volume(node_id, volume).await
    }

    async fn set_node_mute(&self, node_id: u32, mute: bool) -> String {
        self.controller.set_node_mute(node_id, mute).await
    }

    async fn create_link(&self, output_node_id: u32, input_node_id: u32) -> String {
        self.routing.create_link(output_node_id, input_node_id).await
    }

    async fn remove_link(&self, link_id: u32) -> String {
        self.routing.remove_link(link_id).await
    }

    async fn create_virtual_sink(&self, name: String, channels: u32) -> String {
        self.virtual_devices.create_virtual_sink(name, channels).await
    }

    async fn get_swarm_routing_policy(&self) -> String {
        self.routing.get_swarm_routing_policy().await
    }

    async fn set_swarm_routing_policy(&self, policy_json: String) -> String {
        self.routing.set_swarm_routing_policy(policy_json).await
    }
}
