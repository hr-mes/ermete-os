#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};

/// Telemetry metrics for the userspace isolated TCP/IP stack.
#[derive(Debug, Default)]
pub struct NetworkMetrics {
    pub rx_packets: AtomicU64,
    pub tx_packets: AtomicU64,
    pub rx_bytes: AtomicU64,
    pub tx_bytes: AtomicU64,
    pub tcp_connections: AtomicU64,
    pub udp_datagrams: AtomicU64,
    pub dropped_packets: AtomicU64,
    pub active_microvms: AtomicU64,
}

impl NetworkMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc_rx(&self, bytes: u64) {
        self.rx_packets.fetch_add(1, Ordering::Relaxed);
        self.rx_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn inc_tx(&self, bytes: u64) {
        self.tx_packets.fetch_add(1, Ordering::Relaxed);
        self.tx_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn inc_tcp_conn(&self) {
        self.tcp_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_udp(&self) {
        self.udp_datagrams.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_dropped(&self) {
        self.dropped_packets.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_active_microvms(&self, count: u64) {
        self.active_microvms.store(count, Ordering::Relaxed);
    }

    pub fn summary(&self) -> String {
        format!(
            "Rx: {} pkts ({} B), Tx: {} pkts ({} B), TCP Conns: {}, UDP: {}, Dropped: {}, Active Micro-VMs: {}",
            self.rx_packets.load(Ordering::Relaxed),
            self.rx_bytes.load(Ordering::Relaxed),
            self.tx_packets.load(Ordering::Relaxed),
            self.tx_bytes.load(Ordering::Relaxed),
            self.tcp_connections.load(Ordering::Relaxed),
            self.udp_datagrams.load(Ordering::Relaxed),
            self.dropped_packets.load(Ordering::Relaxed),
            self.active_microvms.load(Ordering::Relaxed),
        )
    }
}
