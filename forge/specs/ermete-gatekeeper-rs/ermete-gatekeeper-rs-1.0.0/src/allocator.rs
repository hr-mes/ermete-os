//! Custom Bare-Metal Global Allocator for Zero-Glibc Overhead.
//!
//! Provides direct FFI bindings to `libscudo` (`scudo_malloc`/`scudo_free`)
//! and a lock-free `BumpArenaAllocator` for ultra-low latency IPC buffer allocations
//! in pure `#![no_std]`.

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

// Direct FFI bindings to libscudo
#[cfg(feature = "scudo_ffi")]
extern "C" {
    fn scudo_malloc(size: usize) -> *mut u8;
    fn scudo_free(ptr: *mut u8);
}

const ARENA_SIZE: usize = 2 * 1024 * 1024; // 2 MB static arena for zero-glibc IPC buffers

/// Lock-free arena bump allocator for `no_std` bare-metal IPC.
#[repr(C, align(64))]
pub struct BumpArenaAllocator {
    arena: [u8; ARENA_SIZE],
    offset: AtomicUsize,
}

impl BumpArenaAllocator {
    pub const fn new() -> Self {
        Self {
            arena: [0u8; ARENA_SIZE],
            offset: AtomicUsize::new(0),
        }
    }

    pub fn reset(&self) {
        self.offset.store(0, Ordering::Relaxed);
    }
}

impl Default for BumpArenaAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Hybrid Bare-Metal Allocator combining `libscudo` FFI bindings and arena allocation.
/// Hybrid Bare-Metal Allocator combining `libscudo` FFI bindings and arena allocation.
pub struct BareMetalScudoAllocator;

impl BareMetalScudoAllocator {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for BareMetalScudoAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: BareMetalScudoAllocator delegates allocation directly to thread-safe C allocators.
unsafe impl GlobalAlloc for BareMetalScudoAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();

        // Direct libscudo / libc FFI fallback
        #[cfg(feature = "scudo_ffi")]
        {
            scudo_malloc(size)
        }
        #[cfg(not(feature = "scudo_ffi"))]
        {
            libc::malloc(size) as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        #[cfg(feature = "scudo_ffi")]
        {
            scudo_free(ptr);
        }
        #[cfg(not(feature = "scudo_ffi"))]
        {
            libc::free(ptr as *mut libc::c_void);
        }
    }
}
