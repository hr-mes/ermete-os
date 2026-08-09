//! Ermete OS Mesh Bus Protocol Module
//!
//! Handles binary packet format specification, memory-mapped header layouts,
//! and zero-copy packet extraction directly on AF_XDP UMEM memory.

pub mod zero_copy;

pub use zero_copy::{
    MeshFlags, MeshHeader, MeshMessageType, ZeroCopyFrame, ZeroCopyParser, MESH_MAGIC_BYTES,
    PROTOCOL_VERSION_1,
};
