//! Ermete OS Post-Quantum Mesh Bus — IPC Storage Bridge (Fase 11)
//!
//! Connects the mesh synchronization daemon to the local `ermete-store` storage daemon
//! via a zero-copy lock-free SPSC Shared Memory Ring Buffer (`ZeroCopyRingBuffer`).

use crate::ipc::shm_ring::ZeroCopyRingBuffer;
use crate::sync::crdt_delta::CrdtNetworkPayload;
use anyhow::{anyhow, Context, Result};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Default POSIX Shared Memory path name for IPC ring buffer between mesh bus and storage daemon.
pub const DEFAULT_STORAGE_SHM_NAME: &str = "/ermete_store_sync_ring";

/// Usable ring buffer capacity (2MB shared memory buffer)
pub const DEFAULT_STORAGE_RING_CAPACITY: usize = 2 * 1024 * 1024;

/// IPC Frame command IDs for storage communication
pub const IPC_CMD_CRDT_MERGE: u16 = 0x0701;
pub const IPC_CMD_CRDT_BROADCAST_REQ: u16 = 0x0702;
pub const IPC_CMD_CRDT_ACK: u16 = 0x0703;

/// Interface bridge between PQC Mesh Sync Engine and `ermete-store` Storage Daemon.
pub struct StorageBridge {
    ring_buffer: Arc<ZeroCopyRingBuffer>,
    shm_name: String,
}

impl StorageBridge {
    /// Creates or connects to the named shared memory IPC ring buffer.
    pub fn new(shm_name: Option<&str>, capacity: Option<usize>) -> Result<Self> {
        let name = shm_name.unwrap_or(DEFAULT_STORAGE_SHM_NAME);
        let cap = capacity.unwrap_or(DEFAULT_STORAGE_RING_CAPACITY);

        info!("Initializing IPC Storage Bridge over POSIX shared memory '{}'...", name);

        let ring_buffer = match ZeroCopyRingBuffer::create_named(name, cap) {
            Ok(rb) => Arc::new(rb),
            Err(err) => {
                info!(
                    "POSIX shm creation notice for '{}': {}. Attempting attachment to existing buffer...",
                    name, err
                );
                let attached = ZeroCopyRingBuffer::open_named(name)
                    .with_context(|| format!("Failed to attach to existing SHM ring buffer '{}'", name))?;
                Arc::new(attached)
            }
        };

        info!("IPC Storage Bridge successfully established (capacity: {} bytes)", ring_buffer.capacity());

        Ok(Self {
            ring_buffer,
            shm_name: name.to_string(),
        })
    }

    /// Access inner ring buffer handle
    pub fn ring_buffer(&self) -> &Arc<ZeroCopyRingBuffer> {
        &self.ring_buffer
    }

    /// Access shared memory ring name
    pub fn shm_name(&self) -> &str {
        &self.shm_name
    }

    /// Transmits a `merge` instruction for a received CRDT delta to the local `ermete-store` daemon.
    ///
    /// The instruction is packed into a discrete IPC frame `[IPC_CMD_CRDT_MERGE]` and pushed into
    /// the SPSC shared memory ring buffer without heap lock contention.
    pub fn send_merge_instruction(&self, delta: &CrdtNetworkPayload) -> Result<()> {
        let bytes = delta.serialize()?;

        match self.ring_buffer.push_frame(IPC_CMD_CRDT_MERGE, &bytes) {
            Ok(frame_len) => {
                info!(
                    "Dispatched CRDT merge instruction to local storage daemon (origin: '{}', namespace: '{}', seq: {}, frame_len: {} bytes)",
                    delta.origin_node_id,
                    delta.target_namespace,
                    delta.sequence,
                    frame_len
                );
                Ok(())
            }
            Err(err) => {
                error!(
                    "Failed to push CRDT merge instruction into SHM ring buffer: {}",
                    err
                );
                Err(anyhow!("Storage IPC Ring Buffer push error: {}", err))
            }
        }
    }

    /// Polls the IPC ring buffer for outgoing broadcast requests produced by the local `ermete-store` daemon.
    pub fn receive_broadcast_request(&self) -> Result<Option<CrdtNetworkPayload>> {
        match self.ring_buffer.pop_frame()? {
            Some((IPC_CMD_CRDT_BROADCAST_REQ, payload)) => {
                let delta = CrdtNetworkPayload::deserialize(&payload)?;
                Ok(Some(delta))
            }
            Some((other_cmd, _)) => {
                warn!("Received unexpected IPC frame command {:#X} on storage bridge ring", other_cmd);
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Spawns an asynchronous background worker task that consumes incoming CRDT network deltas
    /// from an MPSC channel and forwards `merge` instructions to `ermete-store` via the SHM ring buffer.
    pub fn spawn_background_merge_dispatcher(
        self: Arc<Self>,
        mut delta_rx: mpsc::Receiver<CrdtNetworkPayload>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("Background CRDT Storage Merge Dispatcher task activated.");
            while let Some(delta) = delta_rx.recv().await {
                if let Err(err) = self.send_merge_instruction(&delta) {
                    error!(
                        "Background merge dispatch error for node '{}' (seq: {}): {}",
                        delta.origin_node_id, delta.sequence, err
                    );
                }
            }
            info!("Background CRDT Storage Merge Dispatcher task terminated.");
        })
    }
}
