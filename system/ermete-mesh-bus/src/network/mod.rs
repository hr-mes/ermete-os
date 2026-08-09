//! Ermete OS Network Engine Module
//!
//! Provides Kernel Bypass networking capabilities via AF_XDP (XSK),
//! enabling high-throughput, zero-copy packet ingestion for post-quantum mesh communications.

pub mod af_xdp;

pub use af_xdp::{AfXdpConfig, AfXdpSocket, AfXdpStats, XdpZeroCopyPacket};
