#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Virtio-Net feature flags for zero-copy Micro-VM hardware offloading
pub struct VirtioNetFlags;

impl VirtioNetFlags {
    pub const VIRTIO_NET_F_CSUM: u64 = 1 << 0;
    pub const VIRTIO_NET_F_GUEST_CSUM: u64 = 1 << 1;
    pub const VIRTIO_NET_F_MAC: u64 = 1 << 5;
    pub const VIRTIO_NET_F_GSO: u64 = 1 << 6;
    pub const VIRTIO_NET_F_GUEST_TSO4: u64 = 1 << 7;
    pub const VIRTIO_NET_F_GUEST_TSO6: u64 = 1 << 8;
    pub const VIRTIO_NET_F_HOST_TSO4: u64 = 1 << 11;
    pub const VIRTIO_NET_F_HOST_TSO6: u64 = 1 << 12;
    pub const VIRTIO_NET_F_MRG_RXBUF: u64 = 1 << 15;
    pub const VIRTIO_NET_F_STATUS: u64 = 1 << 16;
    pub const VIRTIO_NET_F_CTRL_VQ: u64 = 1 << 17;
    pub const VIRTIO_NET_F_MQ: u64 = 1 << 22;
}

/// Virtio-Net packet header (12 bytes when VIRTIO_NET_F_MRG_RXBUF is enabled)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct VirtioNetHeader {
    pub flags: u8,
    pub gso_type: u8,
    pub hdr_len: u16,
    pub gso_size: u16,
    pub csum_start: u16,
    pub csum_offset: u16,
    pub num_buffers: u16,
}

impl VirtioNetHeader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_bytes(&self) -> [u8; 12] {
        let mut buf = [0u8; 12];
        buf[0] = self.flags;
        buf[1] = self.gso_type;
        buf[2..4].copy_from_slice(&self.hdr_len.to_ne_bytes());
        buf[4..6].copy_from_slice(&self.gso_size.to_ne_bytes());
        buf[6..8].copy_from_slice(&self.csum_start.to_ne_bytes());
        buf[8..10].copy_from_slice(&self.csum_offset.to_ne_bytes());
        buf[10..12].copy_from_slice(&self.num_buffers.to_ne_bytes());
        buf
    }
}

/// Metadata descriptor for Micro-VM shared memory ring queue frames
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroVmNetFrame {
    pub vm_id: String,
    pub vq_index: u16,
    pub header: VirtioNetHeader,
    pub payload: Vec<u8>,
}

impl MicroVmNetFrame {
    pub fn new(vm_id: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            vm_id: vm_id.into(),
            vq_index: 0,
            header: VirtioNetHeader::new(),
            payload,
        }
    }
}
