#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unsafe_code)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod allocator;
pub mod security;
pub mod ipc;

pub use allocator::BareMetalScudoAllocator;
pub use security::*;
pub use ipc::*;
