#![allow(clippy::result_unit_err)]
#![allow(clippy::new_without_default)]
pub mod systems;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SharedEcsWorld(Arc<RwLock<()>>);

impl SharedEcsWorld {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(())))
    }
    
    pub fn write(&self) -> Result<std::sync::RwLockWriteGuard<'_, ()>, ()> {
        self.0.write().map_err(|_| ())
    }
}

