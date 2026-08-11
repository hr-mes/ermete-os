//! Bare-Metal IPC Protocol & Zero-Copy Packet Engine (`no_std`).
//!
//! Provides zero-overhead IPC packet structures, Fanotify metadata header parsing,
//! and lock-free IPC buffer processing without glibc or std library dependencies.

use core::mem::size_of;
use crate::security::SecurityError;

/// Magic bytes for Ermete IPC framing ("ERMI")
pub const IPC_MAGIC: [u8; 4] = [0x45, 0x52, 0x4D, 0x49];

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct IpcHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub command: u16,
    pub payload_len: u32,
    pub token_fd_id: u64,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct FanotifyRawEventMetadata {
    pub event_len: u32,
    pub vers: u8,
    pub reserved: u8,
    pub metadata_len: u16,
    pub mask: u64,
    pub fd: i32,
    pub pid: i32,
}

/// Zero-copy fanotify IPC buffer parser operating entirely in `no_std`.
pub struct FanotifyBufferParser<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> FanotifyBufferParser<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, offset: 0 }
    }

    pub fn next_event(&mut self) -> Option<Result<FanotifyRawEventMetadata, SecurityError>> {
        if self.offset >= self.buffer.len() {
            return None;
        }

        let remaining = self.buffer.len() - self.offset;
        if remaining < size_of::<FanotifyRawEventMetadata>() {
            return Some(Err(SecurityError::BufferTooSmall));
        }

        // SAFETY: `self.offset` is bounded by `self.buffer.len()` and we just checked `remaining >= size_of::<FanotifyRawEventMetadata>()`, so the pointer arithmetic is safely within bounds.
        let header_ptr = unsafe {
            self.buffer.as_ptr().add(self.offset) as *const FanotifyRawEventMetadata
        };
        // SAFETY: header_ptr is guaranteed to be a valid pointer to a packed struct returned by the kernel eBPF map.
        let event = unsafe { core::ptr::read_unaligned(header_ptr) };

        if (event.event_len as usize) < size_of::<FanotifyRawEventMetadata>() {
            return Some(Err(SecurityError::InvalidEventLength));
        }

        match self.offset.checked_add(event.event_len as usize) {
            Some(next_off) => {
                if next_off > self.buffer.len() {
                    Some(Err(SecurityError::BufferTooSmall))
                } else {
                    self.offset = next_off;
                    Some(Ok(event))
                }
            }
            None => Some(Err(SecurityError::Overflow)),
        }
    }
}

/// Fast bare-metal serialization for IPC headers without allocation.
pub fn encode_ipc_header(cmd: u16, payload_len: u32, fd_id: u64, out_buf: &mut [u8]) -> Result<usize, SecurityError> {
    let header_size = 20; // 4 + 2 + 2 + 4 + 8
    if out_buf.len() < header_size {
        return Err(SecurityError::BufferTooSmall);
    }

    // Safe serialization without unsafe pointer arithmetic or raw copying
    out_buf[0..4].copy_from_slice(&IPC_MAGIC);
    out_buf[4..6].copy_from_slice(&1u16.to_ne_bytes());
    out_buf[6..8].copy_from_slice(&cmd.to_ne_bytes());
    out_buf[8..12].copy_from_slice(&payload_len.to_ne_bytes());
    out_buf[12..20].copy_from_slice(&fd_id.to_ne_bytes());

    Ok(header_size)
}
