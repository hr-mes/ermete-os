#![allow(unsafe_code)]
#![allow(clippy::multiple_unsafe_ops_per_block)]
#![allow(clippy::undocumented_unsafe_blocks)]
//! Ermete OS Post-Quantum Mesh Bus - AF_XDP Zero-Copy Frame Parser
//!
//! Provides ultra-low latency, zero-allocation binary protocol parsing directly
//! over Linux AF_XDP UMEM packet buffers. Operates without heap allocations (Vec/clone)
//! by performing checked transmutes and raw slice lifetime scoping.

use std::fmt;
use std::ptr;
use anyhow::{anyhow, bail, Result};

/// Magic bytes identifying a valid Ermete OS Post-Quantum Mesh Frame (`ERMQ`)
pub const MESH_MAGIC_BYTES: [u8; 4] = [0x45, 0x52, 0x4D, 0x51]; // "ERMQ"

/// Current protocol layout version
pub const PROTOCOL_VERSION_1: u16 = 1;

/// Packet message types for Mesh Bus communications
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshMessageType {
    HandshakeInit = 0x01,
    HandshakeResponse = 0x02,
    DataFrame = 0x03,
    Heartbeat = 0x04,
    KeyExchange = 0x05,
    ControlSignal = 0x06,
    CrdtSyncFrame = 0x07,
    Unknown = 0xFF,
}

impl From<u8> for MeshMessageType {
    fn from(val: u8) -> Self {
        match val {
            0x01 => MeshMessageType::HandshakeInit,
            0x02 => MeshMessageType::HandshakeResponse,
            0x03 => MeshMessageType::DataFrame,
            0x04 => MeshMessageType::Heartbeat,
            0x05 => MeshMessageType::KeyExchange,
            0x06 => MeshMessageType::ControlSignal,
            0x07 => MeshMessageType::CrdtSyncFrame,
            _ => MeshMessageType::Unknown,
        }
    }
}

/// Post-Quantum Mesh Frame Flags
pub struct MeshFlags;
impl MeshFlags {
    pub const ENCRYPTED: u8   = 1 << 0; // 0x01: Payload encrypted with AEAD
    pub const PQC_SIGNED: u8  = 1 << 1; // 0x02: Post-Quantum Dilithium5 signature present
    pub const COMPRESSED: u8  = 1 << 2; // 0x04: Payload compressed
    pub const UMEM_DIRECT: u8 = 1 << 3; // 0x08: Direct AF_XDP UMEM pass-through
}

/// Binary Memory Layout of the Ermete PQC Mesh Header.
/// Using `#[repr(C, packed)]` guarantees deterministic byte alignment across architectures
/// without padding bytes, enabling safe casting directly over AF_XDP raw network buffers.
#[repr(C, packed)]
pub struct MeshHeader {
    /// 4-byte magic signature ("ERMQ")
    pub magic_bytes: [u8; 4],
    /// Protocol version (big-endian)
    pub version: u16,
    /// Message type enum value (`MeshMessageType`)
    pub msg_type: u8,
    /// Feature & status flags (`MeshFlags`)
    pub flags: u8,
    /// Sequence counter for replay protection & order tracking (big-endian)
    pub sequence: u64,
    /// 32-byte Sender Node Identifier (Ed25519/X25519 hash or node ID)
    pub sender_node_id: [u8; 32],
    /// 32-byte Target Recipient Node Identifier
    pub recipient_node_id: [u8; 32],
    /// 12-byte AEAD cipher Nonce
    pub nonce: [u8; 12],
    /// 64-byte Post-Quantum Cryptographic Auth Signature (Dilithium5 / Kyber HMAC)
    pub kyber_signature: [u8; 64],
    /// Payload length in bytes (big-endian u32)
    pub payload_len: u32,
    /// Header & payload integrity checksum (CRC32/SHA256 truncated)
    pub checksum: u32,
}

impl MeshHeader {
    /// Total fixed size in bytes of the header in memory (160 bytes)
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Reads magic bytes from the header without unaligned reference issues
    #[inline(always)]
    pub fn magic_bytes(&self) -> [u8; 4] {
        unsafe { ptr::read_unaligned(ptr::addr_of!(self.magic_bytes)) }
    }

    /// Reads protocol version converting from big-endian
    #[inline(always)]
    pub fn version(&self) -> u16 {
        let raw = unsafe { ptr::read_unaligned(ptr::addr_of!(self.version)) };
        u16::from_be(raw)
    }

    /// Reads message type enum
    #[inline(always)]
    pub fn msg_type(&self) -> MeshMessageType {
        let raw = unsafe { ptr::read_unaligned(ptr::addr_of!(self.msg_type)) };
        MeshMessageType::from(raw)
    }

    /// Reads message type raw u8
    #[inline(always)]
    pub fn raw_msg_type(&self) -> u8 {
        unsafe { ptr::read_unaligned(ptr::addr_of!(self.msg_type)) }
    }

    /// Reads flags byte
    #[inline(always)]
    pub fn flags(&self) -> u8 {
        unsafe { ptr::read_unaligned(ptr::addr_of!(self.flags)) }
    }

    /// Reads sequence counter converting from big-endian
    #[inline(always)]
    pub fn sequence(&self) -> u64 {
        let raw = unsafe { ptr::read_unaligned(ptr::addr_of!(self.sequence)) };
        u64::from_be(raw)
    }

    /// Reads sender node ID array
    #[inline(always)]
    pub fn sender_node_id(&self) -> [u8; 32] {
        unsafe { ptr::read_unaligned(ptr::addr_of!(self.sender_node_id)) }
    }

    /// Reads recipient node ID array
    #[inline(always)]
    pub fn recipient_node_id(&self) -> [u8; 32] {
        unsafe { ptr::read_unaligned(ptr::addr_of!(self.recipient_node_id)) }
    }

    /// Reads 12-byte cipher nonce
    #[inline(always)]
    pub fn nonce(&self) -> [u8; 12] {
        unsafe { ptr::read_unaligned(ptr::addr_of!(self.nonce)) }
    }

    /// Reads 64-byte Kyber/Dilithium signature array
    #[inline(always)]
    pub fn kyber_signature(&self) -> [u8; 64] {
        unsafe { ptr::read_unaligned(ptr::addr_of!(self.kyber_signature)) }
    }

    /// Reads payload length converting from big-endian
    #[inline(always)]
    pub fn payload_len(&self) -> u32 {
        let raw = unsafe { ptr::read_unaligned(ptr::addr_of!(self.payload_len)) };
        u32::from_be(raw)
    }

    /// Reads header checksum converting from big-endian
    #[inline(always)]
    pub fn checksum(&self) -> u32 {
        let raw = unsafe { ptr::read_unaligned(ptr::addr_of!(self.checksum)) };
        u32::from_be(raw)
    }

    /// Validates magic bytes and basic version sanity
    pub fn validate_header(&self) -> Result<()> {
        let magic = self.magic_bytes();
        if magic != MESH_MAGIC_BYTES {
            bail!(
                "Invalid header magic bytes: {:?}, expected: {:?}",
                magic,
                MESH_MAGIC_BYTES
            );
        }
        let ver = self.version();
        if ver != PROTOCOL_VERSION_1 {
            bail!(
                "Unsupported protocol version: {}, expected: {}",
                ver,
                PROTOCOL_VERSION_1
            );
        }
        Ok(())
    }
}

impl fmt::Debug for MeshHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MeshHeader")
            .field("magic_bytes", &self.magic_bytes())
            .field("version", &self.version())
            .field("msg_type", &self.msg_type())
            .field("flags", &self.flags())
            .field("sequence", &self.sequence())
            .field("sender_node_id", &hex::encode(self.sender_node_id()))
            .field("recipient_node_id", &hex::encode(self.recipient_node_id()))
            .field("nonce", &hex::encode(self.nonce()))
            .field("payload_len", &self.payload_len())
            .field("checksum", &self.checksum())
            .finish()
    }
}

/// Borrowed zero-copy view over an AF_XDP UMEM packet buffer frame.
/// Contains references to the header and the payload slice directly pointing inside UMEM.
/// Zero memory allocations (`Vec`) or cloning occur when accessing fields or payload.
#[derive(Debug, Copy, Clone)]
pub struct ZeroCopyFrame<'a> {
    /// Zero-copy reference to header struct overlaid on raw memory
    pub header: &'a MeshHeader,
    /// Zero-copy borrow slice of packet payload directly from UMEM buffer
    pub payload: &'a [u8],
}

impl<'a> ZeroCopyFrame<'a> {
    /// Returns reference to payload slice without any allocation or cloning.
    #[inline(always)]
    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }

    /// Returns header reference
    #[inline(always)]
    pub fn header(&self) -> &'a MeshHeader {
        self.header
    }

    /// Convenience getter for payload length
    #[inline(always)]
    pub fn payload_len(&self) -> usize {
        self.payload.len()
    }
}

/// Zero-Copy Network Packet Parser for AF_XDP UMEM Buffers.
pub struct ZeroCopyParser;

impl ZeroCopyParser {
    /// Casts raw byte slice from AF_XDP buffer directly into `ZeroCopyFrame<'a>`.
    /// Performs strict bound checks BEFORE performing pointer cast to ensure memory safety.
    ///
    /// # Errors
    /// Returns `Err` if buffer is shorter than `MeshHeader::SIZE`, if magic bytes do not match,
    /// or if declared `payload_len` exceeds remaining buffer size.
    pub fn parse_frame<'a>(buffer: &'a [u8]) -> Result<ZeroCopyFrame<'a>> {
        let header_size = MeshHeader::SIZE;
        if buffer.len() < header_size {
            bail!(
                "AF_XDP buffer size underflow: buffer len {} < required header size {}",
                buffer.len(),
                header_size
            );
        }

        // Safe pointer cast after explicit bounds check
        let header_ptr = buffer.as_ptr() as *const MeshHeader;
        let header: &'a MeshHeader = unsafe { &*header_ptr };

        // Validate header magic and version
        header.validate_header()?;

        let payload_len = header.payload_len() as usize;
        let total_required = header_size.checked_add(payload_len).ok_or_else(|| {
            anyhow!("Payload length overflow calculation")
        })?;

        if buffer.len() < total_required {
            bail!(
                "AF_XDP frame payload truncated: buffer len {} < total required {}",
                buffer.len(),
                total_required
            );
        }

        let payload = &buffer[header_size..total_required];

        Ok(ZeroCopyFrame { header, payload })
    }

    /// Extracts ONLY the payload slice zero-copy from an AF_XDP buffer without cloning.
    #[inline(always)]
    pub fn extract_payload<'a>(buffer: &'a [u8]) -> Result<&'a [u8]> {
        let frame = Self::parse_frame(buffer)?;
        Ok(frame.payload)
    }

    /// Writes a complete `MeshHeader` zero-copy into a mutable byte buffer (e.g. AF_XDP TX UMEM chunk).
    /// Returns mutable slice pointing to payload area inside the target buffer.
    pub fn write_header_zero_copy<'a>(
        target_buffer: &'a mut [u8],
        msg_type: MeshMessageType,
        flags: u8,
        sequence: u64,
        sender_node_id: [u8; 32],
        recipient_node_id: [u8; 32],
        nonce: [u8; 12],
        kyber_signature: [u8; 64],
        payload_len: u32,
        checksum: u32,
    ) -> Result<&'a mut [u8]> {
        let header_size = MeshHeader::SIZE;
        let total_needed = header_size
            .checked_add(payload_len as usize)
            .ok_or_else(|| anyhow!("TX payload buffer size overflow"))?;

        if target_buffer.len() < total_needed {
            bail!(
                "AF_XDP TX UMEM buffer overflow: buffer capacity {} < required {}",
                target_buffer.len(),
                total_needed
            );
        }

        let header_ptr = target_buffer.as_mut_ptr() as *mut MeshHeader;
        unsafe {
            ptr::write_unaligned(
                ptr::addr_of_mut!((*header_ptr).magic_bytes),
                MESH_MAGIC_BYTES,
            );
            ptr::write_unaligned(
                ptr::addr_of_mut!((*header_ptr).version),
                PROTOCOL_VERSION_1.to_be(),
            );
            ptr::write_unaligned(
                ptr::addr_of_mut!((*header_ptr).msg_type),
                msg_type as u8,
            );
            ptr::write_unaligned(ptr::addr_of_mut!((*header_ptr).flags), flags);
            ptr::write_unaligned(
                ptr::addr_of_mut!((*header_ptr).sequence),
                sequence.to_be(),
            );
            ptr::write_unaligned(
                ptr::addr_of_mut!((*header_ptr).sender_node_id),
                sender_node_id,
            );
            ptr::write_unaligned(
                ptr::addr_of_mut!((*header_ptr).recipient_node_id),
                recipient_node_id,
            );
            ptr::write_unaligned(ptr::addr_of_mut!((*header_ptr).nonce), nonce);
            ptr::write_unaligned(
                ptr::addr_of_mut!((*header_ptr).kyber_signature),
                kyber_signature,
            );
            ptr::write_unaligned(
                ptr::addr_of_mut!((*header_ptr).payload_len),
                payload_len.to_be(),
            );
            ptr::write_unaligned(
                ptr::addr_of_mut!((*header_ptr).checksum),
                checksum.to_be(),
            );
        }

        let payload_end = header_size + (payload_len as usize);
        Ok(&mut target_buffer[header_size..payload_end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_parsing_and_payload_borrowing() {
        let mut buffer = vec![0u8; 512];
        let raw_payload = b"POST_QUANTUM_ZERO_COPY_PAYLOAD_TEST";
        let sender = [0x11; 32];
        let recipient = [0x22; 32];
        let nonce = [0x33; 12];
        let sig = [0x44; 64];

        let payload_slice = ZeroCopyParser::write_header_zero_copy(
            &mut buffer,
            MeshMessageType::DataFrame,
            MeshFlags::ENCRYPTED | MeshFlags::UMEM_DIRECT,
            1042,
            sender,
            recipient,
            nonce,
            sig,
            raw_payload.len() as u32,
            0xABCD1234,
        )
        .expect("write_header_zero_copy failed");

        payload_slice.copy_from_slice(raw_payload);

        // Parse zero copy frame
        let frame = ZeroCopyParser::parse_frame(&buffer).expect("parse_frame failed");
        assert_eq!(frame.header().magic_bytes(), MESH_MAGIC_BYTES);
        assert_eq!(frame.header().version(), PROTOCOL_VERSION_1);
        assert_eq!(frame.header().msg_type(), MeshMessageType::DataFrame);
        assert_eq!(frame.header().sequence(), 1042);
        assert_eq!(frame.header().sender_node_id(), sender);
        assert_eq!(frame.header().recipient_node_id(), recipient);
        assert_eq!(frame.header().nonce(), nonce);
        assert_eq!(frame.header().kyber_signature(), sig);
        assert_eq!(frame.header().payload_len(), raw_payload.len() as u32);
        assert_eq!(frame.payload(), raw_payload);
    }

    #[test]
    fn test_underflow_and_magic_validation() {
        let small_buf = [0u8; 10];
        assert!(ZeroCopyParser::parse_frame(&small_buf).is_err());

        let mut invalid_magic_buf = vec![0u8; MeshHeader::SIZE + 10];
        invalid_magic_buf[0..4].copy_from_slice(b"XXXX");
        assert!(ZeroCopyParser::parse_frame(&invalid_magic_buf).is_err());
    }
}
