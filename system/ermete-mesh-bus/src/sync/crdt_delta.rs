//! Ermete OS Post-Quantum Mesh Bus — CRDT Network Payload Definitions (Fase 11)
//!
//! Provides binary-serializable CRDT delta types exchanged between Ermete OS nodes
//! across the zero-trust post-quantum network mesh.

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

/// Categories of CRDT state deltas transmitted over post-quantum mesh frames.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrdtDeltaType {
    /// Full convergent state snapshot synchronization
    FullStateSync,
    /// LWW-Register scalar field update
    RegisterUpdate,
    /// Add-Wins OR-Set element addition
    SetAdd,
    /// Add-Wins OR-Set element tombstone removal
    SetRemove,
    /// System configuration key-value pair mutation
    SettingUpdate,
}

/// Network envelope for CRDT state synchronization frames over AF_XDP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtNetworkPayload {
    /// Sender Node Identifier string (e.g. "node-alpha")
    pub origin_node_id: String,
    /// Target database namespace or table identifier (e.g. "ermete-store", "system-config")
    pub target_namespace: String,
    /// Monotonic sequence counter for order tracking and duplicate suppression
    pub sequence: u64,
    /// Physical timestamp in milliseconds since UNIX epoch
    pub timestamp_ms: u64,
    /// Specific delta operation type
    pub delta_type: CrdtDeltaType,
    /// Encoded CRDT payload (bincode / serde_json of CrdtState or specific mutation)
    pub payload_bytes: Vec<u8>,
    /// Post-Quantum Dilithium5 signature of the origin node over payload_bytes
    pub pqc_signature: Vec<u8>,
}

impl CrdtNetworkPayload {
    /// Creates a new `CrdtNetworkPayload` with timestamp initialization.
    pub fn new(
        origin_node_id: impl Into<String>,
        target_namespace: impl Into<String>,
        sequence: u64,
        delta_type: CrdtDeltaType,
        payload_bytes: Vec<u8>,
        pqc_signature: Vec<u8>,
    ) -> Self {
        let timestamp_ms = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => dur.as_millis() as u64,
            Err(_) => 0,
        };

        Self {
            origin_node_id: origin_node_id.into(),
            target_namespace: target_namespace.into(),
            sequence,
            timestamp_ms,
            delta_type,
            payload_bytes,
            pqc_signature,
        }
    }

    /// Serializes payload struct to binary bytes using `serde_json`.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| anyhow!("Failed to serialize CrdtNetworkPayload: {}", e))
    }

    /// Deserializes binary bytes to `CrdtNetworkPayload`.
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| anyhow!("Failed to deserialize CrdtNetworkPayload: {}", e))
    }

    /// Validates basic integrity (non-empty node ID, valid payload length, signature presence).
    pub fn validate_zero_trust_envelope(&self) -> Result<()> {
        if self.origin_node_id.trim().is_empty() {
            bail!("Zero-Trust Violation: CRDT payload has empty origin_node_id");
        }
        if self.payload_bytes.is_empty() {
            bail!("Zero-Trust Violation: CRDT payload_bytes is empty");
        }
        if self.pqc_signature.is_empty() {
            bail!("Zero-Trust Violation: Missing PQC signature on CRDT payload");
        }
        Ok(())
    }
}
