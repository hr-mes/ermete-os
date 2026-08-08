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
pub struct BareMetalScudoAllocator {
    arena: BumpArenaAllocator,
}

impl BareMetalScudoAllocator {
    pub const fn new() -> Self {
        Self {
            arena: BumpArenaAllocator::new(),
        }
    }
}

impl Default for BareMetalScudoAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: BareMetalScudoAllocator manages allocation via BumpArena in a thread-safe manner using atomic operations.
unsafe impl GlobalAlloc for BareMetalScudoAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();

        // Fast path: align and allocate from static IPC arena (zero glibc overhead)
        let start = self.arena.arena.as_ptr() as usize;
        let mut current = self.arena.offset.load(Ordering::Relaxed);
        loop {
            let ptr_val = (start + current + align - 1) & !(align - 1);
            let offset_needed = ptr_val - start;
            if offset_needed.saturating_add(size) > ARENA_SIZE {
                break;
            }
            match self.arena.offset.compare_exchange_weak(
                current,
                offset_needed + size,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return ptr_val as *mut u8,
                Err(actual) => current = actual,
            }
        }

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
        let arena_start = self.arena.arena.as_ptr() as usize;
        let arena_end = arena_start + ARENA_SIZE;
        let ptr_val = ptr as usize;

        // Arena allocations are zero-overhead (no individual free required)
        if ptr_val >= arena_start && ptr_val < arena_end {
            return;
        }

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
