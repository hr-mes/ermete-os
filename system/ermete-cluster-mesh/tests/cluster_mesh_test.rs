use ermete_cluster_mesh::npu_scheduler::NpuScheduler;
use ermete_cluster_mesh::types::{NpuCapabilities, SwarmNode, SwarmNodeState};
use std::collections::HashMap;

#[tokio::test]
async fn test_npu_scheduler_layer_topology_calculation() {
    let local_caps = NpuCapabilities {
        device_name: "Snapdragon X Elite NPU".to_string(),
        backend: "Hexagon-NPU".to_string(),
        tops: 45.0,
        total_vram_mb: 16384,
        free_vram_mb: 12288,
        supports_llama3_2: true,
        cpu_impact_percentage: 0.0,
    };

    let scheduler = NpuScheduler::new("local-node-1".to_string(), local_caps);

    let mut peers = HashMap::new();
    let peer_caps = NpuCapabilities {
        device_name: "Intel Core Ultra NPU".to_string(),
        backend: "Vulkan-NPU".to_string(),
        tops: 30.0,
        total_vram_mb: 8192,
        free_vram_mb: 6144,
        supports_llama3_2: true,
        cpu_impact_percentage: 0.0,
    };

    peers.insert(
        "peer-node-2".to_string(),
        SwarmNode {
            node_id: "peer-node-2".to_string(),
            hostname: "laptop-ermete".to_string(),
            endpoint_ip: "192.168.1.50".to_string(),
            ipc_port: 51823,
            virtual_ip: Some("10.99.0.2".to_string()),
            dilithium_pk_b64: "pk1".to_string(),
            kyber_pk_b64: "pk2".to_string(),
            x25519_pk_b64: "pk3".to_string(),
            npu_caps: peer_caps,
            state: SwarmNodeState::SwarmIpcActive,
            assigned_layer_range: (0, 0),
            last_seen_secs: 100,
            pqc_verified: true,
        },
    );

    let topology = scheduler.compute_topology(&peers);
    assert_eq!(topology.len(), 2);

    // Total TOPS = 45 + 30 = 75 TOPS
    // Local Node (45 TOPS) -> 45/75 * 28 = 16.8 -> 17 layers (0..16)
    // Peer Node (30 TOPS)  -> 30/75 * 28 = 11.2 -> 11 layers (17..27)
    let total_assigned_layers: u32 = topology.iter().map(|t| t.layer_end - t.layer_start + 1).sum();
    assert_eq!(total_assigned_layers, 28);
}
