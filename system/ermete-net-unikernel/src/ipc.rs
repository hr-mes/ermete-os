#![allow(unsafe_code)]
//! Zero-Copy Lock-Free Shared Memory Ring Buffer for Ermete OS Network Unikernel IPC
//! (Blind Mode Communication Channel between UI and Network Daemon)

use anyhow::{anyhow, Context, Result};
use libc::{
    c_void, ftruncate, mmap, munmap, shm_open, shm_unlink, MAP_SHARED, O_CREAT, O_EXCL, O_RDWR,
    PROT_READ, PROT_WRITE,
};
use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

pub const RING_BUFFER_MAGIC: u64 = 0x4552_4D54_5348_4D31;
pub const FLAG_ACTIVE: u32 = 0x0001;

#[repr(C)]
pub struct RingBufferHeader {
    pub magic: u64,
    pub capacity: usize,
    pub head: AtomicUsize,
    _pad_head: [u8; 56],
    pub tail: AtomicUsize,
    _pad_tail: [u8; 56],
    pub flags: AtomicU32,
    _pad_flags: [u8; 60],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    pub payload_len: u32,
    pub frame_type: u16,
    pub flags: u16,
}

pub struct ZeroCopyRingBuffer {
    fd: RawFd,
    ptr: NonNull<u8>,
    total_size: usize,
    capacity: usize,
    is_owner: bool,
    shm_name: Option<String>,
}

unsafe impl Send for ZeroCopyRingBuffer {}
unsafe impl Sync for ZeroCopyRingBuffer {}

impl ZeroCopyRingBuffer {
    pub fn header_size() -> usize {
        std::mem::size_of::<RingBufferHeader>()
    }

    pub fn create_anonymous(name: &str, capacity: usize) -> Result<Self> {
        let c_name = CString::new(name).context("Invalid name for memfd_create")?;
        let fd = unsafe { libc::memfd_create(c_name.as_ptr(), libc::MFD_CLOEXEC) };
        if fd < 0 {
            return Err(anyhow::Error::from(std::io::Error::last_os_error())).context("memfd_create failed");
        }
        Self::init_from_fd(fd, capacity, true, None)
    }

    pub fn create_named(name: &str, capacity: usize) -> Result<Self> {
        let formatted_name = if name.starts_with('/') {
            name.to_string()
        } else {
            format!("/{}", name)
        };
        let c_name = CString::new(formatted_name.clone()).context("Invalid POSIX shm name")?;
        let fd = unsafe { shm_open(c_name.as_ptr(), O_CREAT | O_RDWR | O_EXCL, 0o660) };
        if fd < 0 {
            return Err(anyhow::Error::from(std::io::Error::last_os_error()))
                .context(format!("shm_open failed for creation of '{}'", formatted_name));
        }
        Self::init_from_fd(fd, capacity, true, Some(formatted_name))
    }

    fn init_from_fd(
        fd: RawFd,
        capacity: usize,
        is_owner: bool,
        shm_name: Option<String>,
    ) -> Result<Self> {
        let total_size = Self::header_size() + capacity;
        let trunc_res = unsafe { ftruncate(fd, total_size as libc::off_t) };
        if trunc_res < 0 {
            unsafe { libc::close(fd) };
            return Err(anyhow::Error::from(std::io::Error::last_os_error())).context("ftruncate failed");
        }
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
            return Err(anyhow::Error::from(std::io::Error::last_os_error())).context("mmap failed");
        }
        let ptr = NonNull::new(mapped as *mut u8)
            .ok_or_else(|| anyhow!("mmap returned null pointer"))?;

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

        Ok(Self {
            fd,
            ptr,
            total_size,
            capacity,
            is_owner,
            shm_name,
        })
    }

    #[inline]
    fn header(&self) -> &RingBufferHeader {
        unsafe { &*(self.ptr.as_ptr() as *const RingBufferHeader) }
    }

    #[inline]
    fn data_ptr(&self) -> *mut u8 {
        unsafe { self.ptr.as_ptr().add(Self::header_size()) }
    }

    pub fn available_write(&self) -> usize {
        let head = self.header().head.load(Ordering::Relaxed);
        let tail = self.header().tail.load(Ordering::Acquire);
        let occupied = head.wrapping_sub(tail);
        self.capacity.saturating_sub(occupied)
    }

    pub fn available_read(&self) -> usize {
        let head = self.header().head.load(Ordering::Acquire);
        let tail = self.header().tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }

    pub fn push(&self, data: &[u8]) -> Result<usize> {
        let len = data.len();
        if len == 0 {
            return Ok(0);
        }
        let avail = self.available_write();
        if avail < len {
            return Err(anyhow!("Ring buffer full (requested {} bytes, available {} bytes)", len, avail));
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

        self.header().head.fetch_add(len, Ordering::Release);
        Ok(len)
    }

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

        self.header().tail.fetch_add(read_len, Ordering::Release);
        Ok(read_len)
    }

    pub fn push_frame(&self, frame_type: u16, data: &[u8]) -> Result<usize> {
        let frame_header = FrameHeader {
            payload_len: data.len() as u32,
            frame_type,
            flags: 0,
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
            return Err(anyhow!("Cannot push frame: insufficient space"));
        }
        self.push(header_bytes)?;
        self.push(data)?;
        Ok(total_frame_len)
    }

    pub fn pop_frame(&self) -> Result<Option<(u16, Vec<u8>)>> {
        let header_size = std::mem::size_of::<FrameHeader>();
        let avail = self.available_read();
        if avail < header_size {
            return Ok(None);
        }

        let mut header_buf = [0u8; std::mem::size_of::<FrameHeader>()];
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
            return Ok(None);
        }

        self.header().tail.fetch_add(header_size, Ordering::Release);
        let mut payload = vec![0u8; frame_header.payload_len as usize];
        if frame_header.payload_len > 0 {
            self.pop(&mut payload)?;
        }
        Ok(Some((frame_header.frame_type, payload)))
    }
}

impl Drop for ZeroCopyRingBuffer {
    fn drop(&mut self) {
        unsafe {
            munmap(self.ptr.as_ptr() as *mut c_void, self.total_size);
            libc::close(self.fd);
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
