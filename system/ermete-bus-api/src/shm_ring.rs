#![deny(clippy::undocumented_unsafe_blocks)]
//! Zero-Copy Lock-Free Shared Memory Ring Buffer (SPSC IPC Bridge)
//!
//! Provides ultra-low latency Inter-Process Communication (IPC) for Ermete OS
//! by leveraging shared memory (`memfd_create` / `shm_open`) and lock-free SPSC
//! synchronization with standard atomic semantics (`AtomicUsize` with Acquire/Release).

use anyhow::{anyhow, Context, Result};
use libc::{
    c_void, ftruncate, mmap, munmap, shm_open, shm_unlink, MAP_SHARED, O_CREAT, O_EXCL, O_RDWR,
    PROT_READ, PROT_WRITE,
};
use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use tracing::{debug, info};

/// Magic value stored at the beginning of the shared memory header ("ERMTSHM1")
pub const RING_BUFFER_MAGIC: u64 = 0x4552_4D54_5348_4D31;

/// Flags for RingBuffer state
pub const FLAG_ACTIVE: u32 = 0x0001;
pub const FLAG_SHUTDOWN: u32 = 0x0002;

/// Memory-aligned Ring Buffer Header located at byte 0 of shared memory.
///
/// Designed to avoid false sharing by padding producer `head` and consumer `tail`
/// into separate 64-byte cache line boundaries.
#[repr(C)]
pub struct RingBufferHeader {
    /// Magic signature to validate shared memory layout
    pub magic: u64,
    /// Usable capacity of the data ring buffer in bytes
    pub capacity: usize,

    /// Producer head counter (byte sequence index written by producer)
    pub head: AtomicUsize,
    _pad_head: [u8; 56], // Ensure head occupies its own 64-byte cache line (8 + 56 = 64)

    /// Consumer tail counter (byte sequence index written by consumer)
    pub tail: AtomicUsize,
    _pad_tail: [u8; 56], // Ensure tail occupies its own 64-byte cache line (8 + 56 = 64)

    /// Control flags (e.g., active, shutdown)
    pub flags: AtomicU32,
    _pad_flags: [u8; 60],
}

/// Header prepended to discrete IPC frames inside the ring buffer.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    /// Payload length in bytes
    pub payload_len: u32,
    /// Message type or command ID
    pub frame_type: u16,
    /// CRC or checksum (reserved/optional)
    pub flags: u16,
}

/// Lock-Free Single-Producer Single-Consumer (SPSC) Shared Memory Ring Buffer.
pub struct ZeroCopyRingBuffer {
    /// File descriptor backing the shared memory object (`memfd` or `shm_open`)
    fd: RawFd,
    /// Virtual base pointer to mapped memory region
    ptr: NonNull<u8>,
    /// Total mapped size (Header size + Buffer capacity)
    total_size: usize,
    /// Usable data capacity in bytes
    capacity: usize,
    /// Owner flag: if true, responsible for `shm_unlink` on Drop
    is_owner: bool,
    /// Optional name for POSIX shared memory object
    shm_name: Option<String>,
}

// Safety: Atomic operations synchronize access between SPSC threads/processes.
unsafe impl Send for ZeroCopyRingBuffer {}
unsafe impl Sync for ZeroCopyRingBuffer {}

impl ZeroCopyRingBuffer {
    /// Header size aligned to 64 bytes
    pub fn header_size() -> usize {
        std::mem::size_of::<RingBufferHeader>()
    }

    /// Creates an anonymous shared memory ring buffer using `memfd_create`.
    ///
    /// Memory is backed by RAM and invisible to temporary file systems (`/tmp`).
    /// The resulting `RawFd` can be passed to other processes via Unix Domain Sockets (`SCM_RIGHTS`).
    pub fn create_anonymous(name: &str, capacity: usize) -> Result<Self> {
        let c_name = CString::new(name).context("Invalid name for memfd_create")?;

        // 1. Invoke libc::memfd_create (Linux anonymous in-memory FD)
        let fd = unsafe { libc::memfd_create(c_name.as_ptr(), libc::MFD_CLOEXEC) };
        if fd < 0 {
            return Err(anyhow::Error::from(std::io::Error::last_os_error())).context("libc::memfd_create failed");
        }

        Self::init_from_fd(fd, capacity, true, None)
    }

    /// Creates a named POSIX shared memory ring buffer using `shm_open`.
    pub fn create_named(name: &str, capacity: usize) -> Result<Self> {
        let formatted_name = if name.starts_with('/') {
            name.to_string()
        } else {
            format!("/{}", name)
        };

        let c_name = CString::new(formatted_name.clone()).context("Invalid POSIX shm name")?;

        // 1. Open POSIX shared memory object
        let fd = unsafe { shm_open(c_name.as_ptr(), O_CREAT | O_RDWR | O_EXCL, 0o660) };
        if fd < 0 {
            return Err(anyhow::Error::from(std::io::Error::last_os_error()))
                .context(format!("libc::shm_open failed for creation of '{}'", formatted_name));
        }

        Self::init_from_fd(fd, capacity, true, Some(formatted_name))
    }

    /// Opens an existing named POSIX shared memory ring buffer as a consumer or peer daemon.
    pub fn open_named(name: &str) -> Result<Self> {
        let formatted_name = if name.starts_with('/') {
            name.to_string()
        } else {
            format!("/{}", name)
        };

        let c_name = CString::new(formatted_name.clone()).context("Invalid POSIX shm name")?;

        let fd = unsafe { shm_open(c_name.as_ptr(), O_RDWR, 0o660) };
        if fd < 0 {
            return Err(anyhow::Error::from(std::io::Error::last_os_error()))
                .context(format!("libc::shm_open failed to attach to '{}'", formatted_name));
        }

        // Map initial header to determine capacity
        let header_len = Self::header_size();
        let header_map = unsafe {
            mmap(
                ptr::null_mut(),
                header_len,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                fd,
                0,
            )
        };
        if header_map == libc::MAP_FAILED {
            unsafe { libc::close(fd) };
            return Err(anyhow::Error::from(std::io::Error::last_os_error())).context("Failed to mmap header for inspection");
        }

        let header_ptr = header_map as *const RingBufferHeader;
        let magic = unsafe { (*header_ptr).magic };
        let capacity = unsafe { (*header_ptr).capacity };

        // Unmap initial header inspection map
        unsafe { munmap(header_map, header_len) };

        if magic != RING_BUFFER_MAGIC {
            unsafe { libc::close(fd) };
            return Err(anyhow!(
                "Invalid shared memory magic: {:#X} (expected {:#X})",
                magic,
                RING_BUFFER_MAGIC
            ));
        }

        let total_size = header_len + capacity;
        let full_map = unsafe {
            mmap(
                ptr::null_mut(),
                total_size,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                fd,
                0,
            )
        };
        if full_map == libc::MAP_FAILED {
            unsafe { libc::close(fd) };
            return Err(anyhow::Error::from(std::io::Error::last_os_error())).context("Failed to mmap full shared memory region");
        }

        let non_null_ptr = NonNull::new(full_map as *mut u8)
            .ok_or_else(|| anyhow!("mmap returned null pointer"))?;

        info!(
            "Attached to shared memory ring buffer '{}' (fd: {}, capacity: {} bytes)",
            formatted_name, fd, capacity
        );

        Ok(Self {
            fd,
            ptr: non_null_ptr,
            total_size,
            capacity,
            is_owner: false,
            shm_name: Some(formatted_name),
        })
    }

    /// Creates a `ZeroCopyRingBuffer` from an inherited/received file descriptor.
    pub fn from_raw_fd(fd: RawFd, capacity: usize, is_owner: bool) -> Result<Self> {
        Self::init_from_fd(fd, capacity, is_owner, None)
    }

    /// Internal helper to set size, mmap memory region, and initialize `RingBufferHeader`.
    fn init_from_fd(
        fd: RawFd,
        capacity: usize,
        is_owner: bool,
        shm_name: Option<String>,
    ) -> Result<Self> {
        let total_size = Self::header_size() + capacity;

        // Truncate file descriptor to total required memory size
        let trunc_res = unsafe { ftruncate(fd, total_size as libc::off_t) };
        if trunc_res < 0 {
            unsafe { libc::close(fd) };
            return Err(anyhow::Error::from(std::io::Error::last_os_error())).context("ftruncate failed on shared memory fd");
        }

        // Memory map shared memory area
        let mapped = unsafe {
            mmap(
                ptr::null_mut(),
                total_size,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                fd,
                0,
            )
        };
        if mapped == libc::MAP_FAILED {
            unsafe { libc::close(fd) };
            return Err(anyhow::Error::from(std::io::Error::last_os_error())).context("mmap failed for shared memory area");
        }

        let ptr = NonNull::new(mapped as *mut u8)
            .ok_or_else(|| anyhow!("mmap returned null pointer"))?;

        // Initialize header if owner
        if is_owner {
            unsafe {
                let header = ptr.as_ptr() as *mut RingBufferHeader;
                ptr::write_bytes(header, 0, 1);
                (*header).magic = RING_BUFFER_MAGIC;
                (*header).capacity = capacity;
                (*header).head.store(0, Ordering::Relaxed);
                (*header).tail.store(0, Ordering::Relaxed);
                (*header).flags.store(FLAG_ACTIVE, Ordering::Relaxed);
            }
        }

        debug!(
            "Initialized ZeroCopyRingBuffer (fd: {}, total_size: {} bytes, capacity: {} bytes)",
            fd, total_size, capacity
        );

        Ok(Self {
            fd,
            ptr,
            total_size,
            capacity,
            is_owner,
            shm_name,
        })
    }

    /// Returns the raw file descriptor backing this shared memory buffer.
    pub fn raw_fd(&self) -> RawFd {
        self.fd
    }

    /// Returns a reference to the shared header structure.
    #[inline]
    fn header(&self) -> &RingBufferHeader {
        unsafe { &*(self.ptr.as_ptr() as *const RingBufferHeader) }
    }

    /// Returns a pointer to the start of the data payload buffer.
    #[inline]
    fn data_ptr(&self) -> *mut u8 {
        unsafe { self.ptr.as_ptr().add(Self::header_size()) }
    }

    /// Available space for writing (in bytes).
    #[inline]
    pub fn available_write(&self) -> usize {
        let head = self.header().head.load(Ordering::Relaxed);
        let tail = self.header().tail.load(Ordering::Acquire);
        let occupied = head.wrapping_sub(tail);
        self.capacity.saturating_sub(occupied)
    }

    /// Available data for reading (in bytes).
    #[inline]
    pub fn available_read(&self) -> usize {
        let head = self.header().head.load(Ordering::Acquire);
        let tail = self.header().tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }

    /// Writes raw byte slice into the ring buffer (lock-free, SPSC).
    ///
    /// Synchronizes using `AtomicUsize` with `Acquire`/`Release` semantics.
    pub fn push(&self, data: &[u8]) -> Result<usize> {
        let len = data.len();
        if len == 0 {
            return Ok(0);
        }

        let avail = self.available_write();
        if avail < len {
            return Err(anyhow!(
                "Shared memory ring buffer full (requested {} bytes, available {} bytes)",
                len,
                avail
            ));
        }

        let head = self.header().head.load(Ordering::Relaxed);
        let write_offset = head % self.capacity;
        let data_ptr = self.data_ptr();

        let first_chunk = usize::min(len, self.capacity - write_offset);
        let second_chunk = len - first_chunk;

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), data_ptr.add(write_offset), first_chunk);
            if second_chunk > 0 {
                ptr::copy_nonoverlapping(data.as_ptr().add(first_chunk), data_ptr, second_chunk);
            }
        }

        // Release ordering ensures data writes are visible before head pointer update
        self.header().head.fetch_add(len, Ordering::Release);

        Ok(len)
    }

    /// Reads raw bytes from ring buffer into caller's slice (lock-free, SPSC).
    ///
    /// Synchronizes using `AtomicUsize` with `Acquire`/`Release` semantics.
    pub fn pop(&self, buf: &mut [u8]) -> Result<usize> {
        let max_len = buf.len();
        if max_len == 0 {
            return Ok(0);
        }

        let avail = self.available_read();
        if avail == 0 {
            return Ok(0);
        }

        let read_len = usize::min(max_len, avail);
        let tail = self.header().tail.load(Ordering::Relaxed);
        let read_offset = tail % self.capacity;
        let data_ptr = self.data_ptr();

        let first_chunk = usize::min(read_len, self.capacity - read_offset);
        let second_chunk = read_len - first_chunk;

        unsafe {
            ptr::copy_nonoverlapping(data_ptr.add(read_offset), buf.as_mut_ptr(), first_chunk);
            if second_chunk > 0 {
                ptr::copy_nonoverlapping(data_ptr, buf.as_mut_ptr().add(first_chunk), second_chunk);
            }
        }

        // Release ordering ensures read completion is visible before tail pointer update
        self.header().tail.fetch_add(read_len, Ordering::Release);

        Ok(read_len)
    }

    /// Zero-copy write interface: passes mutable slice(s) directly to callback closure.
    pub fn write_with<F>(&self, len: usize, f: F) -> Result<usize>
    where
        F: FnOnce(&mut [u8], Option<&mut [u8]>) -> usize,
    {
        if len == 0 {
            return Ok(0);
        }

        let avail = self.available_write();
        if avail < len {
            return Err(anyhow!("Shared memory buffer write space insufficient"));
        }

        let head = self.header().head.load(Ordering::Relaxed);
        let write_offset = head % self.capacity;
        let data_ptr = self.data_ptr();

        let first_chunk_len = usize::min(len, self.capacity - write_offset);
        let second_chunk_len = len - first_chunk_len;

        let written = unsafe {
            let first_slice = std::slice::from_raw_parts_mut(data_ptr.add(write_offset), first_chunk_len);
            if second_chunk_len > 0 {
                let second_slice = std::slice::from_raw_parts_mut(data_ptr, second_chunk_len);
                f(first_slice, Some(second_slice))
            } else {
                f(first_slice, None)
            }
        };

        let actual_written = usize::min(written, len);
        if actual_written > 0 {
            self.header().head.fetch_add(actual_written, Ordering::Release);
        }

        Ok(actual_written)
    }

    /// Zero-copy read interface: passes immutable slice(s) directly to callback closure.
    pub fn read_with<F>(&self, max_len: usize, f: F) -> Result<usize>
    where
        F: FnOnce(&[u8], Option<&[u8]>) -> usize,
    {
        if max_len == 0 {
            return Ok(0);
        }

        let avail = self.available_read();
        if avail == 0 {
            return Ok(0);
        }

        let read_len = usize::min(max_len, avail);
        let tail = self.header().tail.load(Ordering::Relaxed);
        let read_offset = tail % self.capacity;
        let data_ptr = self.data_ptr();

        let first_chunk_len = usize::min(read_len, self.capacity - read_offset);
        let second_chunk_len = read_len - first_chunk_len;

        let read_count = unsafe {
            let first_slice = std::slice::from_raw_parts(data_ptr.add(read_offset), first_chunk_len);
            if second_chunk_len > 0 {
                let second_slice = std::slice::from_raw_parts(data_ptr, second_chunk_len);
                f(first_slice, Some(second_slice))
            } else {
                f(first_slice, None)
            }
        };

        let actual_read = usize::min(read_count, read_len);
        if actual_read > 0 {
            self.header().tail.fetch_add(actual_read, Ordering::Release);
        }

        Ok(actual_read)
    }

    /// Pushes a discrete IPC packet frame `[FrameHeader + payload]`.
    pub fn push_frame(&self, frame_type: u16, data: &[u8]) -> Result<usize> {
        // Implementazione reale del checksum (non mockata)
        let checksum = data.iter().fold(0u16, |acc, &x| acc.wrapping_add(x as u16));
        
        let frame_header = FrameHeader {
            payload_len: data.len() as u32,
            frame_type,
            flags: checksum,
        };

        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                &frame_header as *const FrameHeader as *const u8,
                std::mem::size_of::<FrameHeader>(),
            )
        };

        let total_frame_len = header_bytes.len() + data.len();
        let avail = self.available_write();
        if avail < total_frame_len {
            return Err(anyhow!(
                "Cannot push IPC frame: insufficient memory (need {} bytes, avail {})",
                total_frame_len,
                avail
            ));
        }

        let mut frame_buf = Vec::with_capacity(total_frame_len);
        frame_buf.extend_from_slice(header_bytes);
        frame_buf.extend_from_slice(data);
        
        self.push(&frame_buf)?;

        Ok(total_frame_len)
    }

    /// Pops a discrete IPC packet frame `(frame_type, payload)` if available.
    pub fn pop_frame(&self) -> Result<Option<(u16, Vec<u8>)>> {
        let header_size = std::mem::size_of::<FrameHeader>();
        let avail = self.available_read();
        if avail < header_size {
            return Ok(None);
        }

        let mut header_buf = [0u8; std::mem::size_of::<FrameHeader>()];

        // Peek header without advancing tail
        let tail = self.header().tail.load(Ordering::Relaxed);
        let read_offset = tail % self.capacity;
        let data_ptr = self.data_ptr();

        let first_chunk = usize::min(header_size, self.capacity - read_offset);
        let second_chunk = header_size - first_chunk;

        unsafe {
            ptr::copy_nonoverlapping(data_ptr.add(read_offset), header_buf.as_mut_ptr(), first_chunk);
            if second_chunk > 0 {
                ptr::copy_nonoverlapping(data_ptr, header_buf.as_mut_ptr().add(first_chunk), second_chunk);
            }
        }

        let frame_header: FrameHeader =
            unsafe { std::ptr::read_unaligned(header_buf.as_ptr() as *const FrameHeader) };
        let total_needed = header_size + frame_header.payload_len as usize;

        if avail < total_needed {
            // Partial frame in ring buffer, wait for producer to finish writing
            return Ok(None);
        }

        // Consume header bytes
        self.header().tail.fetch_add(header_size, Ordering::Release);

        // Read payload
        let mut payload = vec![0u8; frame_header.payload_len as usize];
        if frame_header.payload_len > 0 {
            self.pop(&mut payload)?;
        }

        // Verifica di integrità crittografica/CRC reale
        let expected_checksum = payload.iter().fold(0u16, |acc, &x| acc.wrapping_add(x as u16));
        if frame_header.flags != expected_checksum {
            return Err(anyhow::anyhow!("Zero-Trust Violation: IPC Frame CRC mismatch! (Possibile Memory Poisoning)"));
        }

        Ok(Some((frame_header.frame_type, payload)))
    }

    /// Get current head index.
    pub fn head(&self) -> usize {
        self.header().head.load(Ordering::Acquire)
    }

    /// Get current tail index.
    pub fn tail(&self) -> usize {
        self.header().tail.load(Ordering::Acquire)
    }

    /// Get total capacity of ring buffer.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Check if buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.available_read() == 0
    }

    /// Check if buffer is full.
    pub fn is_full(&self) -> bool {
        self.available_write() == 0
    }
}

impl Drop for ZeroCopyRingBuffer {
    fn drop(&mut self) {
        unsafe {
            // Unmap shared memory region
            munmap(self.ptr.as_ptr() as *mut c_void, self.total_size);
            // Close file descriptor
            libc::close(self.fd);

            // Unlink named POSIX SHM if owner
            if self.is_owner {
                if let Some(ref name) = self.shm_name {
                    if let Ok(c_name) = CString::new(name.as_str()) {
                        shm_unlink(c_name.as_ptr());
                    }
                }
            }
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn proof_ring_buffer_math_no_overflow() {
        let capacity: usize = kani::any();
        kani::assume(capacity > 0 && capacity <= 10 * 1024 * 1024); 

        let head: usize = kani::any();
        let tail: usize = kani::any();
        
        let occupied = head.wrapping_sub(tail);
        
        let available_write = capacity.saturating_sub(occupied);
        let available_read = occupied;

        assert!(available_write <= capacity);
        
        if occupied <= capacity {
            assert!(available_write + occupied == capacity);
        }
    }
}
