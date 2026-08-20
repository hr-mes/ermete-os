//! AF_XDP (XSK) High-Performance Kernel Bypass Engine for Ermete OS Mesh Bus
//!
//! Provides zero-copy packet ingress/egress directly between Network Interface Cards (NICs)
//! and user-space memory (UMEM), completely bypassing the Linux kernel TCP/IP network stack.
//! Enables ultra-low latency post-quantum packet ingestion (ML-KEM-1024 / Dilithium5).

use anyhow::{anyhow, Result};
use libc::{
    c_void, getsockopt, if_nametoindex, mmap, munmap, sendto, setsockopt, socket, MAP_ANONYMOUS,
    MAP_FAILED, MAP_POPULATE, MAP_PRIVATE, MAP_SHARED, MSG_DONTWAIT, PROT_READ, PROT_WRITE,
    SOCK_RAW,
};
use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};

// ============================================================================
// AF_XDP Linux Kernel ABI Constants & FFI Definitions (linux/if_xdp.h)
// ============================================================================

/// Address Family for XDP Sockets
pub const AF_XDP: i32 = 44;
pub const PF_XDP: i32 = AF_XDP;

/// Socket level for AF_XDP setsockopt/getsockopt
pub const SOL_XDP: i32 = 283;

/// Socket options
pub const XDP_MMAP_OFFSETS: i32 = 1;
pub const XDP_RX_RING: i32 = 2;
pub const XDP_TX_RING: i32 = 3;
pub const XDP_UMEM_REG: i32 = 4;
pub const XDP_UMEM_FILL_RING: i32 = 5;
pub const XDP_UMEM_COMPLETION_RING: i32 = 6;
pub const XDP_STATISTICS: i32 = 7;
pub const XDP_OPTIONS: i32 = 8;

/// Mmap offset page magic values
pub const XDP_PGOFF_RX_RING: libc::off_t = 0;
pub const XDP_PGOFF_TX_RING: libc::off_t = 0x80000000;
pub const XDP_UMEM_PGOFF_FILL_RING: libc::off_t = 0x100000000;
pub const XDP_UMEM_PGOFF_COMPL_RING: libc::off_t = 0x180000000;

/// Sockaddr XDP bind flags
pub const XDP_SHARED_UMEM: u16 = 1 << 0;
pub const XDP_COPY: u16 = 1 << 1;
pub const XDP_ZEROCOPY: u16 = 1 << 2;
pub const XDP_USE_NEED_WAKEUP: u16 = 1 << 3;

/// Ring flags from kernel
pub const XDP_RING_NEED_WAKEUP: u32 = 1 << 0;

/// UMEM Registration Descriptor
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct XdpUmemReg {
    pub addr: u64,
    pub len: u64,
    pub chunk_size: u32,
    pub headroom: u32,
    pub flags: u32,
}

/// XDP Rx/Tx Packet Descriptor
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct XdpDesc {
    pub addr: u64,
    pub len: u32,
    pub options: u32,
}

/// Ring Offset Structure
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct XdpRingOffset {
    pub producer: u64,
    pub consumer: u64,
    pub desc: u64,
    pub flags: u64,
}

/// Ring Mmap Offsets Structure returned by getsockopt(XDP_MMAP_OFFSETS)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct XdpMmapOffsets {
    pub rx: XdpRingOffset,
    pub tx: XdpRingOffset,
    pub fr: XdpRingOffset,
    pub cr: XdpRingOffset,
}

/// Socket Address Structure for AF_XDP
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SockAddrXdp {
    pub sxdp_family: u16,
    pub sxdp_flags: u16,
    pub sxdp_ifindex: u32,
    pub sxdp_queue_id: u32,
    pub sxdp_shared_umem_fd: u32,
}

/// XDP Statistics Structure
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct XdpStatistics {
    pub rx_dropped: u64,
    pub rx_invalid_descs: u64,
    pub tx_invalid_descs: u64,
    pub rx_ring_full: u64,
    pub rx_fill_ring_empty: u64,
    pub tx_ring_empty: u64,
}

// ============================================================================
// AF_XDP Configuration & Statistics
// ============================================================================

/// Configuration for AF_XDP Kernel Bypass Socket
#[derive(Debug, Clone)]
pub struct AfXdpConfig {
    pub if_name: String,
    pub queue_id: u32,
    pub frame_size: u32,
    pub frame_count: u32,
    pub rx_ring_size: u32,
    pub tx_ring_size: u32,
    pub fill_ring_size: u32,
    pub comp_ring_size: u32,
    pub zero_copy: bool,
    pub headroom: u32,
}

/// Autodetect active network interface on system (bypassing hardcoded "eth0").
///
/// 1. Inspects `/proc/net/route` for interface with default gateway route (`00000000`).
/// 2. Scans `/sys/class/net/` for operational interface in state `up` or `unknown`.
/// 3. Falls back across standard interface name candidates (`wlan0`, `enp3s0`, `eth0`, `enp0s3`, `end0`).
pub fn detect_active_interface() -> String {
    if let Ok(route_content) = std::fs::read_to_string("/proc/net/route") {
        for line in route_content.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 4 {
                let iface = fields[0];
                let dest = fields[1];
                let flags = fields[3];
                if dest == "00000000" && iface != "lo" {
                    if let Ok(flags_val) = u16::from_str_radix(flags, 16) {
                        if flags_val & 0x0001 != 0 {
                            info!("Autodetected active default route network interface: '{}'", iface);
                            return iface.to_string();
                        }
                    }
                }
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let if_name = entry.file_name().to_string_lossy().to_string();
            if if_name == "lo"
                || if_name.starts_with("veth")
                || if_name.starts_with("docker")
                || if_name.starts_with("br-")
                || if_name.starts_with("virbr")
            {
                continue;
            }
            let operstate_path = entry.path().join("operstate");
            if let Ok(state) = std::fs::read_to_string(&operstate_path) {
                let state_str = state.trim();
                if state_str == "up" || state_str == "unknown" {
                    info!("Autodetected active network interface via sysfs operstate: '{}'", if_name);
                    return if_name;
                }
            }
        }
    }

    let candidates = ["wlan0", "enp3s0", "eth0", "enp0s3", "end0"];
    for &iface in &candidates {
        let sys_path = format!("/sys/class/net/{}", iface);
        if std::path::Path::new(&sys_path).exists() {
            info!("Selected fallback network interface present on system: '{}'", iface);
            return iface.to_string();
        }
    }

    info!("Fallback to default network interface 'eth0'");
    "eth0".to_string()
}

impl Default for AfXdpConfig {
    fn default() -> Self {
        Self {
            if_name: detect_active_interface(),
            queue_id: 0,
            frame_size: 2048,
            frame_count: 4096,
            rx_ring_size: 2048,
            tx_ring_size: 2048,
            fill_ring_size: 2048,
            comp_ring_size: 2048,
            zero_copy: true,
            headroom: 256,
        }
    }
}

/// Socket-level runtime statistics
#[derive(Debug, Default)]
pub struct AfXdpStats {
    pub rx_packets: AtomicU64,
    pub tx_packets: AtomicU64,
    pub rx_bytes: AtomicU64,
    pub tx_bytes: AtomicU64,
    pub rx_dropped: AtomicU64,
    pub invalid_descs: AtomicU64,
}

impl AfXdpStats {
    pub fn snapshot(&self) -> (u64, u64, u64, u64, u64, u64) {
        (
            self.rx_packets.load(Ordering::Relaxed),
            self.tx_packets.load(Ordering::Relaxed),
            self.rx_bytes.load(Ordering::Relaxed),
            self.tx_bytes.load(Ordering::Relaxed),
            self.rx_dropped.load(Ordering::Relaxed),
            self.invalid_descs.load(Ordering::Relaxed),
        )
    }
}

// ============================================================================
// UMEM (User Memory) Shared Memory Allocator
// ============================================================================

/// Thread-safe User Memory (UMEM) buffer mapped directly into NIC DMA address space
pub struct Umem {
    mem: *mut c_void,
    size: usize,
    chunk_size: usize,
    chunk_count: usize,
    headroom: usize,
    freelist: Arc<Mutex<Vec<u64>>>,
}

unsafe impl Send for Umem {}
unsafe impl Sync for Umem {}

impl Umem {
    /// Allocate UMEM memory region via anonymous mmap
    pub fn new(frame_size: usize, frame_count: usize, headroom: usize) -> Result<Self> {
        let size = frame_size * frame_count;
        let mem = unsafe {
            mmap(
                ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_POPULATE,
                -1,
                0,
            )
        };

        if mem == MAP_FAILED {
            return Err(anyhow!("Failed to mmap UMEM memory region of size {} bytes", size));
        }

        let mut freelist = Vec::with_capacity(frame_count);
        for i in 0..frame_count {
            freelist.push((i * frame_size) as u64);
        }

        info!(
            "Allocated AF_XDP UMEM: {} MB ({} frames of {} bytes, {} bytes headroom)",
            size / (1024 * 1024),
            frame_count,
            frame_size,
            headroom
        );

        Ok(Self {
            mem,
            size,
            chunk_size: frame_size,
            chunk_count: frame_count,
            headroom,
            freelist: Arc::new(Mutex::new(freelist)),
        })
    }

    pub fn base_ptr(&self) -> *mut c_void {
        self.mem
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    pub fn headroom(&self) -> usize {
        self.headroom
    }

    /// Allocate a UMEM frame address from freelist
    pub fn alloc_frame(&self) -> Result<u64> {
        let mut list = self
            .freelist
            .lock()
            .map_err(|e| anyhow!("UMEM freelist mutex poisoned: {}", e))?;
        list.pop()
            .ok_or_else(|| anyhow!("AF_XDP UMEM freelist exhausted (out of packet frame buffers)"))
    }

    /// Return a frame address to the freelist
    pub fn free_frame(&self, addr: u64) -> Result<()> {
        let mut list = self
            .freelist
            .lock()
            .map_err(|e| anyhow!("UMEM freelist mutex poisoned: {}", e))?;
        list.push(addr);
        Ok(())
    }

    /// Get raw pointer to frame offset
    pub fn get_slice(&self, addr: u64, len: usize) -> Result<&[u8]> {
        if addr as usize + len > self.size {
            return Err(anyhow!("UMEM frame access out of bounds: addr {} + len {} > size {}", addr, len, self.size));
        }
        unsafe {
            let ptr = (self.mem as *const u8).add(addr as usize);
            Ok(std::slice::from_raw_parts(ptr, len))
        }
    }

    /// Get mutable raw pointer to frame offset
    pub fn get_mut_slice(&self, addr: u64, len: usize) -> Result<&mut [u8]> {
        if addr as usize + len > self.size {
            return Err(anyhow!("UMEM frame access out of bounds: addr {} + len {} > size {}", addr, len, self.size));
        }
        unsafe {
            let ptr = (self.mem as *mut u8).add(addr as usize);
            Ok(std::slice::from_raw_parts_mut(ptr, len))
        }
    }
}

impl Drop for Umem {
    fn drop(&mut self) {
        if !self.mem.is_null() && self.mem != MAP_FAILED {
            unsafe {
                munmap(self.mem, self.size);
            }
            debug!("Unmapped AF_XDP UMEM region of size {} bytes", self.size);
        }
    }
}

// ============================================================================
// Ring Buffers (Rx, Tx, Fill, Completion)
// ============================================================================

/// RX Descriptor Ring Buffer
pub struct RxRing {
    producer: *mut u32,
    consumer: *mut u32,
    flags: *mut u32,
    ring: *mut XdpDesc,
    size: u32,
    mask: u32,
    cached_prod: u32,
}

unsafe impl Send for RxRing {}

impl RxRing {
    pub fn new(map_ptr: *mut c_void, offsets: &XdpRingOffset, size: u32) -> Self {
        unsafe {
            let producer = (map_ptr as *mut u8).add(offsets.producer as usize) as *mut u32;
            let consumer = (map_ptr as *mut u8).add(offsets.consumer as usize) as *mut u32;
            let flags = (map_ptr as *mut u8).add(offsets.flags as usize) as *mut u32;
            let ring = (map_ptr as *mut u8).add(offsets.desc as usize) as *mut XdpDesc;

            Self {
                producer,
                consumer,
                flags,
                ring,
                size,
                mask: size - 1,
                cached_prod: 0,
            }
        }
    }

    /// Peek available descriptors from kernel
    pub fn peek(&mut self, max: u32) -> (u32, u32) {
        unsafe {
            let cons = ptr::read_volatile(self.consumer);
            if self.cached_prod == cons {
                self.cached_prod = ptr::read_volatile(self.producer);
            }
            let avail = self.cached_prod.wrapping_sub(cons);
            let count = avail.min(max);
            (cons, count)
        }
    }

    /// Get descriptor at given consumer index
    pub fn get_desc(&self, index: u32) -> XdpDesc {
        unsafe {
            let idx = (index & self.mask) as usize;
            ptr::read_volatile(self.ring.add(idx))
        }
    }

    /// Advance consumer pointer to release descriptors back to kernel
    pub fn release(&mut self, count: u32) {
        unsafe {
            let cons = ptr::read_volatile(self.consumer);
            ptr::write_volatile(self.consumer, cons.wrapping_add(count));
        }
    }
}

/// TX Descriptor Ring Buffer
pub struct TxRing {
    producer: *mut u32,
    consumer: *mut u32,
    flags: *mut u32,
    ring: *mut XdpDesc,
    size: u32,
    mask: u32,
    cached_cons: u32,
}

unsafe impl Send for TxRing {}

impl TxRing {
    pub fn new(map_ptr: *mut c_void, offsets: &XdpRingOffset, size: u32) -> Self {
        unsafe {
            let producer = (map_ptr as *mut u8).add(offsets.producer as usize) as *mut u32;
            let consumer = (map_ptr as *mut u8).add(offsets.consumer as usize) as *mut u32;
            let flags = (map_ptr as *mut u8).add(offsets.flags as usize) as *mut u32;
            let ring = (map_ptr as *mut u8).add(offsets.desc as usize) as *mut XdpDesc;

            Self {
                producer,
                consumer,
                flags,
                ring,
                size,
                mask: size - 1,
                cached_cons: 0,
            }
        }
    }

    /// Reserve space in TX ring for descriptors
    pub fn reserve(&mut self, count: u32) -> Option<u32> {
        unsafe {
            let prod = ptr::read_volatile(self.producer);
            let free_slots = self.size - (prod.wrapping_sub(self.cached_cons));
            if free_slots < count {
                self.cached_cons = ptr::read_volatile(self.consumer);
                let free_slots_refreshed = self.size - (prod.wrapping_sub(self.cached_cons));
                if free_slots_refreshed < count {
                    return None;
                }
            }
            Some(prod)
        }
    }

    /// Set TX descriptor at producer index
    pub fn set_desc(&mut self, index: u32, desc: XdpDesc) {
        unsafe {
            let idx = (index & self.mask) as usize;
            ptr::write_volatile(self.ring.add(idx), desc);
        }
    }

    /// Submit written descriptors to kernel by advancing producer pointer
    pub fn submit(&mut self, count: u32) {
        unsafe {
            let prod = ptr::read_volatile(self.producer);
            ptr::write_volatile(self.producer, prod.wrapping_add(count));
        }
    }
}

/// Fill Ring Buffer (User passes frame addresses to kernel Rx)
pub struct FillRing {
    producer: *mut u32,
    consumer: *mut u32,
    flags: *mut u32,
    ring: *mut u64,
    size: u32,
    mask: u32,
    cached_cons: u32,
}

unsafe impl Send for FillRing {}

impl FillRing {
    pub fn new(map_ptr: *mut c_void, offsets: &XdpRingOffset, size: u32) -> Self {
        unsafe {
            let producer = (map_ptr as *mut u8).add(offsets.producer as usize) as *mut u32;
            let consumer = (map_ptr as *mut u8).add(offsets.consumer as usize) as *mut u32;
            let flags = (map_ptr as *mut u8).add(offsets.flags as usize) as *mut u32;
            let ring = (map_ptr as *mut u8).add(offsets.desc as usize) as *mut u64;

            Self {
                producer,
                consumer,
                flags,
                ring,
                size,
                mask: size - 1,
                cached_cons: 0,
            }
        }
    }

    /// Fill frames into kernel Rx queue
    pub fn fill(&mut self, addrs: &[u64]) -> u32 {
        if addrs.is_empty() {
            return 0;
        }

        unsafe {
            let count = addrs.len() as u32;
            let prod = ptr::read_volatile(self.producer);
            let free_slots = self.size - (prod.wrapping_sub(self.cached_cons));
            
            let to_fill = if free_slots < count {
                self.cached_cons = ptr::read_volatile(self.consumer);
                (self.size - (prod.wrapping_sub(self.cached_cons))).min(count)
            } else {
                count
            };

            for i in 0..to_fill {
                let idx = ((prod + i) & self.mask) as usize;
                ptr::write_volatile(self.ring.add(idx), addrs[i as usize]);
            }

            ptr::write_volatile(self.producer, prod.wrapping_add(to_fill));
            to_fill
        }
    }
}

/// Completion Ring Buffer (Kernel returns completed Tx frame addresses)
pub struct CompRing {
    producer: *mut u32,
    consumer: *mut u32,
    flags: *mut u32,
    ring: *mut u64,
    size: u32,
    mask: u32,
    cached_prod: u32,
}

unsafe impl Send for CompRing {}

impl CompRing {
    pub fn new(map_ptr: *mut c_void, offsets: &XdpRingOffset, size: u32) -> Self {
        unsafe {
            let producer = (map_ptr as *mut u8).add(offsets.producer as usize) as *mut u32;
            let consumer = (map_ptr as *mut u8).add(offsets.consumer as usize) as *mut u32;
            let flags = (map_ptr as *mut u8).add(offsets.flags as usize) as *mut u32;
            let ring = (map_ptr as *mut u8).add(offsets.desc as usize) as *mut u64;

            Self {
                producer,
                consumer,
                flags,
                ring,
                size,
                mask: size - 1,
                cached_prod: 0,
            }
        }
    }

    /// Reclaim completed Tx frame addresses from kernel
    pub fn reclaim(&mut self, max: u32) -> Vec<u64> {
        unsafe {
            let cons = ptr::read_volatile(self.consumer);
            if self.cached_prod == cons {
                self.cached_prod = ptr::read_volatile(self.producer);
            }

            let avail = self.cached_prod.wrapping_sub(cons);
            let count = avail.min(max);

            let mut reclaimed = Vec::with_capacity(count as usize);
            for i in 0..count {
                let idx = ((cons + i) & self.mask) as usize;
                let addr = ptr::read_volatile(self.ring.add(idx));
                reclaimed.push(addr);
            }

            if count > 0 {
                ptr::write_volatile(self.consumer, cons.wrapping_add(count));
            }

            reclaimed
        }
    }
}

// ============================================================================
// Zero-Copy Packet Representation
// ============================================================================

/// Zero-Copy Packet wrapper pointing to UMEM payload buffer
pub struct XdpZeroCopyPacket {
    umem: Arc<Umem>,
    addr: u64,
    len: u32,
}

impl XdpZeroCopyPacket {
    pub fn new(umem: Arc<Umem>, addr: u64, len: u32) -> Self {
        Self { umem, addr, len }
    }

    pub fn addr(&self) -> u64 {
        self.addr
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Direct zero-copy immutable payload slice
    pub fn payload(&self) -> Result<&[u8]> {
        let offset = self.addr + self.umem.headroom() as u64;
        self.umem.get_slice(offset, self.len as usize)
    }

    /// Raw ethernet frame including headroom
    pub fn raw_frame(&self) -> Result<&[u8]> {
        self.umem.get_slice(self.addr, self.len as usize + self.umem.headroom())
    }
}

impl Drop for XdpZeroCopyPacket {
    fn drop(&mut self) {
        if let Err(e) = self.umem.free_frame(self.addr) {
            error!("Failed to release zero-copy packet frame {}: {}", self.addr, e);
        }
    }
}

// ============================================================================
// Main AF_XDP Socket Manager Structure
// ============================================================================

/// High-Performance AF_XDP Kernel Bypass Socket Engine
pub struct AfXdpSocket {
    fd: RawFd,
    if_index: u32,
    queue_id: u32,
    config: AfXdpConfig,
    umem: Arc<Umem>,
    rx_ring: RxRing,
    tx_ring: TxRing,
    fill_ring: FillRing,
    comp_ring: CompRing,
    rx_map: *mut c_void,
    tx_map: *mut c_void,
    fill_map: *mut c_void,
    comp_map: *mut c_void,
    rx_map_size: usize,
    tx_map_size: usize,
    fill_map_size: usize,
    comp_map_size: usize,
    stats: Arc<AfXdpStats>,
}

unsafe impl Send for AfXdpSocket {}

impl AfXdpSocket {
    /// Create and initialize an AF_XDP socket bound to interface and queue ID
    pub fn new(config: AfXdpConfig) -> Result<Self> {
        // 1. Resolve network interface index
        let c_ifname = CString::new(config.if_name.as_str())
            .map_err(|e| anyhow!("Invalid interface name '{}': {}", config.if_name, e))?;
        
        let if_index = unsafe { if_nametoindex(c_ifname.as_ptr()) };
        if if_index == 0 {
            return Err(anyhow!("Network interface '{}' not found on system", config.if_name));
        }

        // 2. Open Raw AF_XDP Socket
        let fd = unsafe { socket(AF_XDP, SOCK_RAW, 0) };
        if fd < 0 {
            return Err(anyhow!(
                "Failed to open AF_XDP socket (errno: {}). Check root/CAP_NET_RAW privileges.",
                std::io::Error::last_os_error()
            ));
        }

        info!(
            "Opened AF_XDP Socket (fd {}) for interface '{}' (ifindex {}, queue {})",
            fd, config.if_name, if_index, config.queue_id
        );

        // 3. Allocate UMEM Shared Memory
        let umem = Arc::new(Umem::new(
            config.frame_size as usize,
            config.frame_count as usize,
            config.headroom as usize,
        )?);

        // 4. Register UMEM with Kernel via setsockopt
        let umem_reg = XdpUmemReg {
            addr: umem.base_ptr() as u64,
            len: umem.len() as u64,
            chunk_size: config.frame_size,
            headroom: config.headroom,
            flags: 0,
        };

        let res = unsafe {
            setsockopt(
                fd,
                SOL_XDP,
                XDP_UMEM_REG,
                &umem_reg as *const _ as *const c_void,
                std::mem::size_of::<XdpUmemReg>() as u32,
            )
        };
        if res < 0 {
            unsafe { libc::close(fd) };
            return Err(anyhow!("setsockopt(XDP_UMEM_REG) failed: {}", std::io::Error::last_os_error()));
        }

        // 5. Configure Ring Sizes
        unsafe {
            setsockopt(fd, SOL_XDP, XDP_RX_RING, &config.rx_ring_size as *const _ as *const c_void, 4);
            setsockopt(fd, SOL_XDP, XDP_TX_RING, &config.tx_ring_size as *const _ as *const c_void, 4);
            setsockopt(fd, SOL_XDP, XDP_UMEM_FILL_RING, &config.fill_ring_size as *const _ as *const c_void, 4);
            setsockopt(fd, SOL_XDP, XDP_UMEM_COMPLETION_RING, &config.comp_ring_size as *const _ as *const c_void, 4);
        }

        // 6. Fetch MMAP Ring Offsets from Kernel
        let mut offsets = XdpMmapOffsets::default();
        let mut optlen = std::mem::size_of::<XdpMmapOffsets>() as u32;
        let res = unsafe {
            getsockopt(
                fd,
                SOL_XDP,
                XDP_MMAP_OFFSETS,
                &mut offsets as *mut _ as *mut c_void,
                &mut optlen as *mut u32,
            )
        };
        if res < 0 {
            unsafe { libc::close(fd) };
            return Err(anyhow!("getsockopt(XDP_MMAP_OFFSETS) failed: {}", std::io::Error::last_os_error()));
        }

        // 7. Mmap Rings into User-Space Memory
        let fill_map_size = offsets.fr.desc as usize + (config.fill_ring_size as usize * 8);
        let comp_map_size = offsets.cr.desc as usize + (config.comp_ring_size as usize * 8);
        let rx_map_size = offsets.rx.desc as usize + (config.rx_ring_size as usize * std::mem::size_of::<XdpDesc>());
        let tx_map_size = offsets.tx.desc as usize + (config.tx_ring_size as usize * std::mem::size_of::<XdpDesc>());

        let fill_map = unsafe {
            mmap(ptr::null_mut(), fill_map_size, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_POPULATE, fd, XDP_UMEM_PGOFF_FILL_RING)
        };
        let comp_map = unsafe {
            mmap(ptr::null_mut(), comp_map_size, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_POPULATE, fd, XDP_UMEM_PGOFF_COMPL_RING)
        };
        let rx_map = unsafe {
            mmap(ptr::null_mut(), rx_map_size, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_POPULATE, fd, XDP_PGOFF_RX_RING)
        };
        let tx_map = unsafe {
            mmap(ptr::null_mut(), tx_map_size, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_POPULATE, fd, XDP_PGOFF_TX_RING)
        };

        if fill_map == MAP_FAILED || comp_map == MAP_FAILED || rx_map == MAP_FAILED || tx_map == MAP_FAILED {
            unsafe { libc::close(fd) };
            return Err(anyhow!("Failed to mmap AF_XDP ring buffers from kernel"));
        }

        let mut fill_ring = FillRing::new(fill_map, &offsets.fr, config.fill_ring_size);
        let comp_ring = CompRing::new(comp_map, &offsets.cr, config.comp_ring_size);
        let rx_ring = RxRing::new(rx_map, &offsets.rx, config.rx_ring_size);
        let tx_ring = TxRing::new(tx_map, &offsets.tx, config.tx_ring_size);

        // 8. Populate initial Fill Ring frames
        let mut initial_addrs = Vec::with_capacity(config.fill_ring_size as usize);
        for _ in 0..config.fill_ring_size {
            if let Ok(addr) = umem.alloc_frame() {
                initial_addrs.push(addr);
            }
        }
        let filled = fill_ring.fill(&initial_addrs);
        info!("Initialized AF_XDP Fill Ring with {} frame buffers", filled);

        // 9. Bind Socket to Network Interface & Queue ID
        let sxdp_flags = if config.zero_copy { XDP_ZEROCOPY } else { XDP_COPY };
        let sxdp = SockAddrXdp {
            sxdp_family: AF_XDP as u16,
            sxdp_flags,
            sxdp_ifindex: if_index,
            sxdp_queue_id: config.queue_id,
            sxdp_shared_umem_fd: 0,
        };

        let res = unsafe {
            libc::bind(
                fd,
                &sxdp as *const _ as *const libc::sockaddr,
                std::mem::size_of::<SockAddrXdp>() as u32,
            )
        };
        if res < 0 {
            unsafe { libc::close(fd) };
            return Err(anyhow!(
                "Failed to bind AF_XDP socket to interface '{}' (queue {}): {}",
                config.if_name,
                config.queue_id,
                std::io::Error::last_os_error()
            ));
        }

        info!(
            "AF_XDP Kernel Bypass socket initialized successfully on {} (Zero-Copy: {})",
            config.if_name, config.zero_copy
        );

        Ok(Self {
            fd,
            if_index,
            queue_id: config.queue_id,
            config,
            umem,
            rx_ring,
            tx_ring,
            fill_ring,
            comp_ring,
            rx_map,
            tx_map,
            fill_map,
            comp_map,
            rx_map_size,
            tx_map_size,
            fill_map_size,
            comp_map_size,
            stats: Arc::new(AfXdpStats::default()),
        })
    }

    /// Receive a burst of incoming zero-copy packets from Rx ring
    pub fn recv_burst(&mut self, max_pkts: usize) -> Result<Vec<XdpZeroCopyPacket>> {
        let (cons, count) = self.rx_ring.peek(max_pkts as u32);
        if count == 0 {
            return Ok(Vec::new());
        }

        let mut packets = Vec::with_capacity(count as usize);

        for i in 0..count {
            let desc = self.rx_ring.get_desc(cons + i);
            let packet = XdpZeroCopyPacket::new(self.umem.clone(), desc.addr, desc.len);
            packets.push(packet);

            self.stats.rx_packets.fetch_add(1, Ordering::Relaxed);
            self.stats.rx_bytes.fetch_add(desc.len as u64, Ordering::Relaxed);
        }

        self.rx_ring.release(count);
        self.refill_fill_ring()?;

        Ok(packets)
    }

    /// Transmit a packet payload via zero-copy TX ring
    pub fn send_packet(&mut self, payload: &[u8]) -> Result<()> {
        if payload.len() > self.config.frame_size as usize - self.config.headroom as usize {
            return Err(anyhow!("Payload length {} exceeds max frame payload capacity", payload.len()));
        }

        // Reclaim completed TX frames first
        self.reclaim_tx_frames()?;

        // Allocate UMEM frame for transmit
        let frame_addr = self.umem.alloc_frame()?;
        let data_offset = frame_addr + self.config.headroom as u64;

        // Copy payload to UMEM frame
        let slice = self.umem.get_mut_slice(data_offset, payload.len())?;
        slice.copy_from_slice(payload);

        // Reserve slot in TX ring
        let prod = self
            .tx_ring
            .reserve(1)
            .ok_or_else(|| anyhow!("AF_XDP TX ring full"))?;

        let desc = XdpDesc {
            addr: data_offset,
            len: payload.len() as u32,
            options: 0,
        };

        self.tx_ring.set_desc(prod, desc);
        self.tx_ring.submit(1);

        self.stats.tx_packets.fetch_add(1, Ordering::Relaxed);
        self.stats.tx_bytes.fetch_add(payload.len() as u64, Ordering::Relaxed);

        // Kick kernel TX engine if non-blocking push is active
        unsafe {
            sendto(self.fd, ptr::null(), 0, MSG_DONTWAIT, ptr::null(), 0);
        }

        Ok(())
    }

    /// Refill Fill Ring with available UMEM frames
    pub fn refill_fill_ring(&mut self) -> Result<u32> {
        let mut addrs = Vec::new();
        while let Ok(addr) = self.umem.alloc_frame() {
            addrs.push(addr);
            if addrs.len() >= 64 {
                break;
            }
        }

        let filled = self.fill_ring.fill(&addrs);
        // Put un-filled addresses back into freelist
        for addr in &addrs[filled as usize..] {
            let _ = self.umem.free_frame(*addr);
        }

        Ok(filled)
    }

    /// Reclaim completed TX frame addresses from Completion ring back to UMEM freelist
    pub fn reclaim_tx_frames(&mut self) -> Result<usize> {
        let reclaimed = self.comp_ring.reclaim(128);
        let count = reclaimed.len();

        for addr in reclaimed {
            // Restore frame base address
            let base_addr = addr - (addr % self.config.frame_size as u64);
            let _ = self.umem.free_frame(base_addr);
        }

        Ok(count)
    }

    /// Fetch runtime socket statistics
    pub fn stats(&self) -> Arc<AfXdpStats> {
        self.stats.clone()
    }

    /// Raw socket file descriptor
    pub fn raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for AfXdpSocket {
    fn drop(&mut self) {
        unsafe {
            if !self.rx_map.is_null() && self.rx_map != MAP_FAILED {
                munmap(self.rx_map, self.rx_map_size);
            }
            if !self.tx_map.is_null() && self.tx_map != MAP_FAILED {
                munmap(self.tx_map, self.tx_map_size);
            }
            if !self.fill_map.is_null() && self.fill_map != MAP_FAILED {
                munmap(self.fill_map, self.fill_map_size);
            }
            if !self.comp_map.is_null() && self.comp_map != MAP_FAILED {
                munmap(self.comp_map, self.comp_map_size);
            }
            if self.fd >= 0 {
                libc::close(self.fd);
                info!("Closed AF_XDP Socket (fd {})", self.fd);
            }
        }
    }
}
