//! Ermete OS — Post-Quantum Mesh CRDT Synchronization Engine (Fase 11)
//!
//! Provides zero-copy network frame ingestion over AF_XDP for CRDT deltas,
//! zero-trust Dilithium5 signature verification, and background IPC merge dispatching
//! to the local `ermete-store` storage daemon via Shared Memory SPSC Ring Buffers.

pub mod crdt_broadcaster;
pub mod crdt_delta;
pub mod storage_bridge;

pub use crdt_broadcaster::CrdtBroadcaster;
pub use crdt_delta::{CrdtDeltaType, CrdtNetworkPayload};
pub use storage_bridge::{
    StorageBridge, IPC_CMD_CRDT_ACK, IPC_CMD_CRDT_BROADCAST_REQ, IPC_CMD_CRDT_MERGE,
};
