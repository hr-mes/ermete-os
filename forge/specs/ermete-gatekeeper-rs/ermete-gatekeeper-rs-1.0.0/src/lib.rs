#![no_std]
#![allow(unsafe_code)]

extern crate alloc;

pub mod allocator;
pub mod security;
pub mod ipc;

pub use allocator::BareMetalScudoAllocator;
pub use security::*;
pub use ipc::*;
