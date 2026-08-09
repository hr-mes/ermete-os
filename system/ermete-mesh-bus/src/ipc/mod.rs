//! Zero-Latency Shared Memory (SHM) Inter-Process Communication (IPC) Bridge
//!
//! Provides zero-copy SPSC lock-free ring buffer primitives for high-throughput,
//! ultra-low latency IPC between Ermete OS micro-daemons.

pub mod shm_ring;

pub use shm_ring::{
    FrameHeader, RingBufferHeader, ZeroCopyRingBuffer, FLAG_ACTIVE, FLAG_SHUTDOWN,
    RING_BUFFER_MAGIC,
};
