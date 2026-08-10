use anyhow::{anyhow, Result};
use ermete_bus_api::shm_ring::ZeroCopyRingBuffer;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Clone)]
pub struct PqcMeshClient {
    channel_name: String,
    ring_buffer: Arc<Mutex<Option<ZeroCopyRingBuffer>>>,
}

impl PqcMeshClient {
    pub fn new() -> Self {
        let channel_name = "ermete-pqc-mesh-ringbuf".to_string();
        let ring = ZeroCopyRingBuffer::open_named(&channel_name)
            .or_else(|_| ZeroCopyRingBuffer::create_named(&channel_name, 4 * 1024 * 1024))
            .ok();

        Self {
            channel_name,
            ring_buffer: Arc::new(Mutex::new(ring)),
        }
    }

    fn get_or_init_ring_buffer<'a>(
        ring_guard: &'a mut Option<ZeroCopyRingBuffer>,
        channel_name: &str,
    ) -> Result<&'a ZeroCopyRingBuffer> {
        if ring_guard.is_none() {
            let ring = ZeroCopyRingBuffer::open_named(channel_name)
                .or_else(|_| ZeroCopyRingBuffer::create_named(channel_name, 4 * 1024 * 1024))
                .map_err(|e| anyhow!("Failed to open/create PQC ZeroCopyRingBuffer: {}", e))?;
            *ring_guard = Some(ring);
        }
        ring_guard
            .as_ref()
            .ok_or_else(|| anyhow!("ZeroCopyRingBuffer channel unavailable"))
    }

    pub async fn check_status(&self) -> Result<String> {
        let mut guard = self.ring_buffer.lock().await;
        let ring = Self::get_or_init_ring_buffer(&mut guard, &self.channel_name)?;

        let payload = serde_json::json!({
            "command": "status"
        });
        let payload_bytes = serde_json::to_vec(&payload)?;

        ring.push_frame(1, &payload_bytes)?;

        Ok(format!(
            "PQC Mesh Shared Memory RingBuffer Active (channel: '{}', capacity: {} KB, lock-free IPC)",
            self.channel_name,
            ring.capacity() / 1024
        ))
    }

    #[allow(dead_code)]
    pub async fn get_local_identity(&self) -> Result<serde_json::Value> {
        let mut guard = self.ring_buffer.lock().await;
        let ring = Self::get_or_init_ring_buffer(&mut guard, &self.channel_name)?;

        let payload = serde_json::json!({
            "command": "get_node_identity"
        });
        let payload_bytes = serde_json::to_vec(&payload)?;

        ring.push_frame(2, &payload_bytes)?;

        let identity = serde_json::json!({
            "channel": self.channel_name,
            "status": "active",
            "ipc_transport": "ZeroCopyRingBuffer"
        });

        Ok(identity)
    }

    pub async fn register_and_handshake_peer(
        &self,
        node_id: &str,
        endpoint: &str,
        dilithium_pk_b64: &str,
        kyber_pk_b64: &str,
        x25519_pk_b64: &str,
    ) -> Result<()> {
        let mut guard = self.ring_buffer.lock().await;
        let ring = match Self::get_or_init_ring_buffer(&mut guard, &self.channel_name) {
            Ok(r) => r,
            Err(e) => {
                warn!("Unable to access PQC ZeroCopyRingBuffer channel: {}", e);
                return Err(anyhow!("Shared memory ring buffer connection failed: {}", e));
            }
        };

        info!(
            "PqcMeshClient: Registering peer '{}' at '{}' via ZeroCopyRingBuffer low-latency channel",
            node_id, endpoint
        );

        // 1. Push add_peer frame
        let add_peer_payload = serde_json::json!({
            "action": "add_peer",
            "node_id": node_id,
            "endpoint": endpoint,
            "dilithium_pk_b64": dilithium_pk_b64,
            "kyber_pk_b64": kyber_pk_b64,
            "x25519_pk_b64": x25519_pk_b64,
        });
        let add_bytes = serde_json::to_vec(&add_peer_payload)?;
        ring.push_frame(10, &add_bytes)?;
        info!("PQC MeshBus add_peer frame submitted to ZeroCopyRingBuffer for node '{}'", node_id);

        // 2. Push initiate_handshake frame
        let handshake_payload = serde_json::json!({
            "action": "initiate_handshake",
            "node_id": node_id,
            "endpoint": endpoint,
        });
        let hs_bytes = serde_json::to_vec(&handshake_payload)?;
        ring.push_frame(11, &hs_bytes)?;
        info!("PQC MeshBus initiate_handshake frame submitted to ZeroCopyRingBuffer for node '{}'", node_id);

        Ok(())
    }
}

