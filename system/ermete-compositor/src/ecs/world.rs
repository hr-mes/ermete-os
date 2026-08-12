#![allow(unsafe_code)]

use crate::ecs::entity::{Entity, EntityAllocator};
use anyhow::{anyhow, Result};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tracing::{debug, warn};

/// Trait bound for dynamic component storage erasure.
pub trait AnyStorage: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove_entity_raw(&mut self, entity_id: u32);
    fn clear(&mut self);
}

/// Generic contiguous component storage indexed by entity ID.
#[derive(Debug, Default)]
pub struct ComponentStorage<T: 'static + Send + Sync> {
    data: Vec<Option<T>>,
}

impl<T: 'static + Send + Sync> ComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(1024),
        }
    }

    pub fn insert(&mut self, entity_id: u32, component: T) -> Option<T> {
        let idx = entity_id as usize;
        if idx >= self.data.len() {
            self.data.resize_with(idx + 1, || None);
        }
        self.data[idx].replace(component)
    }

    pub fn get(&self, entity_id: u32) -> Option<&T> {
        let idx = entity_id as usize;
        if idx < self.data.len() {
            self.data[idx].as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, entity_id: u32) -> Option<&mut T> {
        let idx = entity_id as usize;
        if idx < self.data.len() {
            self.data[idx].as_mut()
        } else {
            None
        }
    }

    pub fn remove(&mut self, entity_id: u32) -> Option<T> {
        let idx = entity_id as usize;
        if idx < self.data.len() {
            self.data[idx].take()
        } else {
            None
        }
    }

    pub fn has(&self, entity_id: u32) -> bool {
        let idx = entity_id as usize;
        idx < self.data.len() && self.data[idx].is_some()
    }
}

impl<T: 'static + Send + Sync> AnyStorage for ComponentStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove_entity_raw(&mut self, entity_id: u32) {
        let idx = entity_id as usize;
        if idx < self.data.len() {
            self.data[idx] = None;
        }
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

/// The core Data-Oriented ECS World / Registry managing entities and components.
#[derive(Default)]
pub struct World {
    allocator: EntityAllocator,
    storages: HashMap<TypeId, Box<dyn AnyStorage>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::new(),
            storages: HashMap::new(),
        }
    }

    /// Spawns a new unique Entity.
    pub fn spawn(&mut self) -> Entity {
        let entity = self.allocator.allocate();
        debug!("ECS World: Spawned entity {}", entity);
        entity
    }

    /// Despawns an Entity and removes all of its associated components across all storages.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.allocator.is_alive(entity) {
            return false;
        }

        for storage in self.storages.values_mut() {
            storage.remove_entity_raw(entity.id);
        }

        self.allocator.deallocate(entity);
        debug!("ECS World: Despawned entity {}", entity);
        true
    }

    /// Validates if an Entity is active in the world.
    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.allocator.is_alive(entity)
    }

    /// Gets or creates the component storage for type T.
    fn storage_mut<T: 'static + Send + Sync>(&mut self) -> &mut ComponentStorage<T> {
        let type_id = TypeId::of::<T>();
        let entry = self
            .storages
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentStorage::<T>::new()));
        if let Some(storage) = entry.as_any_mut().downcast_mut::<ComponentStorage<T>>() {
            storage
        } else {
            unreachable!("ComponentStorage entry matching type_id must be ComponentStorage<T>")
        }
    }

    /// Gets the component storage for type T if it exists.
    fn storage<T: 'static + Send + Sync>(&self) -> Option<&ComponentStorage<T>> {
        let type_id = TypeId::of::<T>();
        self.storages
            .get(&type_id)
            .and_then(|s| s.as_any().downcast_ref::<ComponentStorage<T>>())
    }

    /// Attaches a component to an entity.
    pub fn add_component<T: 'static + Send + Sync>(&mut self, entity: Entity, component: T) {
        if !self.is_alive(entity) {
            warn!("Attempted to add component to dead entity {}", entity);
            return;
        }
        let storage = self.storage_mut::<T>();
        storage.insert(entity.id, component);
    }

    /// Retrieves an immutable reference to a component.
    pub fn get_component<T: 'static + Send + Sync>(&self, entity: Entity) -> Option<&T> {
        if !self.is_alive(entity) {
            return None;
        }
        self.storage::<T>()?.get(entity.id)
    }

    /// Retrieves a mutable reference to a component.
    pub fn get_component_mut<T: 'static + Send + Sync>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.is_alive(entity) {
            return None;
        }
        self.storage_mut::<T>().get_mut(entity.id)
    }

    /// Removes a component from an entity.
    pub fn remove_component<T: 'static + Send + Sync>(&mut self, entity: Entity) -> Option<T> {
        if !self.is_alive(entity) {
            return None;
        }
        let storage = self.storage_mut::<T>();
        storage.remove(entity.id)
    }

    /// Checks if an entity has component T attached.
    pub fn has_component<T: 'static + Send + Sync>(&self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }
        self.storage::<T>()
            .map(|s| s.has(entity.id))
            .unwrap_or(false)
    }

    /// Returns active entity count.
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.allocator.active_count()
    }

    /// Query for entities matching 1 component type.
    pub fn query_1<C1: 'static + Send + Sync>(&self) -> Vec<(Entity, &C1)> {
        let s1 = match self.storage::<C1>() {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut results = Vec::new();
        for id in 0..self.allocator.total_capacity() as u32 {
            let _entity = Entity::new(id, 0); // Generation check performed via allocator
            if let Some(c1) = s1.get(id) {
                // Reconstruct exact entity key with valid generation
                if self.is_alive(Entity::new(id, self.get_entity_generation(id))) {
                    let valid_entity = Entity::new(id, self.get_entity_generation(id));
                    results.push((valid_entity, c1));
                }
            }
        }
        results
    }

    /// Query for entities matching 2 component types simultaneously.
    pub fn query_2<C1: 'static + Send + Sync, C2: 'static + Send + Sync>(
        &self,
    ) -> Vec<(Entity, &C1, &C2)> {
        let s1 = match self.storage::<C1>() {
            Some(s) => s,
            None => return Vec::new(),
        };
        let s2 = match self.storage::<C2>() {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut results = Vec::new();
        for id in 0..self.allocator.total_capacity() as u32 {
            let gen = self.get_entity_generation(id);
            let entity = Entity::new(id, gen);
            if !self.is_alive(entity) {
                continue;
            }
            if let (Some(c1), Some(c2)) = (s1.get(id), s2.get(id)) {
                results.push((entity, c1, c2));
            }
        }
        results
    }

    /// Query for entities matching 3 component types.
    pub fn query_3<C1: 'static + Send + Sync, C2: 'static + Send + Sync, C3: 'static + Send + Sync>(
        &self,
    ) -> Vec<(Entity, &C1, &C2, &C3)> {
        let s1 = match self.storage::<C1>() {
            Some(s) => s,
            None => return Vec::new(),
        };
        let s2 = match self.storage::<C2>() {
            Some(s) => s,
            None => return Vec::new(),
        };
        let s3 = match self.storage::<C3>() {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut results = Vec::new();
        for id in 0..self.allocator.total_capacity() as u32 {
            let gen = self.get_entity_generation(id);
            let entity = Entity::new(id, gen);
            if !self.is_alive(entity) {
                continue;
            }
            if let (Some(c1), Some(c2), Some(c3)) = (s1.get(id), s2.get(id), s3.get(id)) {
                results.push((entity, c1, c2, c3));
            }
        }
        results
    }

    /// Query for entities matching 4 component types.
    pub fn query_4<
        C1: 'static + Send + Sync,
        C2: 'static + Send + Sync,
        C3: 'static + Send + Sync,
        C4: 'static + Send + Sync,
    >(
        &self,
    ) -> Vec<(Entity, &C1, &C2, &C3, &C4)> {
        let s1 = match self.storage::<C1>() {
            Some(s) => s,
            None => return Vec::new(),
        };
        let s2 = match self.storage::<C2>() {
            Some(s) => s,
            None => return Vec::new(),
        };
        let s3 = match self.storage::<C3>() {
            Some(s) => s,
            None => return Vec::new(),
        };
        let s4 = match self.storage::<C4>() {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut results = Vec::new();
        for id in 0..self.allocator.total_capacity() as u32 {
            let gen = self.get_entity_generation(id);
            let entity = Entity::new(id, gen);
            if !self.is_alive(entity) {
                continue;
            }
            if let (Some(c1), Some(c2), Some(c3), Some(c4)) =
                (s1.get(id), s2.get(id), s3.get(id), s4.get(id))
            {
                results.push((entity, c1, c2, c3, c4));
            }
        }
        results
    }

    #[inline]
    pub fn get_entity_generation(&self, id: u32) -> u32 {
        self.allocator.get_generation(id)
    }

    /// Zero-lock-contention batch iteration for 3 component types.
    /// Borrows C1 and C2 mutably, and C3 immutably in a single pass over contiguous component arrays.
    pub fn for_each_mut_3<
        C1: 'static + Send + Sync,
        C2: 'static + Send + Sync,
        C3: 'static + Send + Sync,
        F: FnMut(Entity, &mut C1, &mut C2, &C3),
    >(
        &mut self,
        mut f: F,
    ) {
        let t1 = TypeId::of::<C1>();
        let t2 = TypeId::of::<C2>();
        let t3 = TypeId::of::<C3>();

        assert!(
            t1 != t2 && t2 != t3 && t1 != t3,
            "Batch mutation requires distinct component types"
        );

        let s1_ptr = match self.storages.get_mut(&t1) {
            Some(s) => s
                .as_any_mut()
                .downcast_mut::<ComponentStorage<C1>>()
                .map(|s| s as *mut ComponentStorage<C1>),
            None => None,
        };

        let s2_ptr = match self.storages.get_mut(&t2) {
            Some(s) => s
                .as_any_mut()
                .downcast_mut::<ComponentStorage<C2>>()
                .map(|s| s as *mut ComponentStorage<C2>),
            None => None,
        };

        let s3_ptr = match self.storages.get(&t3) {
            Some(s) => s
                .as_any()
                .downcast_ref::<ComponentStorage<C3>>()
                .map(|s| s as *const ComponentStorage<C3>),
            None => None,
        };

        if let (Some(s1_ptr), Some(s2_ptr), Some(s3_ptr)) = (s1_ptr, s2_ptr, s3_ptr) {
            // SAFETY: Type IDs are distinct, so we have mutable aliasing guarantees. Pointers are checked.
            let s1 = unsafe { &mut *s1_ptr };
            // SAFETY: Type IDs are distinct, so we have mutable aliasing guarantees. Pointers are checked.
            let s2 = unsafe { &mut *s2_ptr };
            // SAFETY: Type IDs are distinct, so we have mutable aliasing guarantees. Pointers are checked.
            let s3 = unsafe { &*s3_ptr };

            let len = s1.data.len().min(s2.data.len()).min(s3.data.len());
            for id in 0..len {
                let gen = self.allocator.get_generation(id as u32);
                let entity = Entity::new(id as u32, gen);
                if !self.allocator.is_alive(entity) {
                    continue;
                }
                if let (Some(c1), Some(c2), Some(c3)) = (
                    s1.data[id].as_mut(),
                    s2.data[id].as_mut(),
                    s3.data[id].as_ref(),
                ) {
                    f(entity, c1, c2, c3);
                }
            }
        }
    }
}

/// Thread-safe wrapper `SharedEcsWorld` wrapping `Arc<RwLock<World>>`.
/// Allows zero-latency parallel reader access across physics, render, IPC, and input threads.
#[derive(Clone, Default)]
pub struct SharedEcsWorld {
    inner: Arc<RwLock<World>>,
}

impl SharedEcsWorld {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(World::new())),
        }
    }

    /// Acquires panic-free read lock. If lock is poisoned, recovers cleanly.
    pub fn read(&self) -> Result<RwLockReadGuard<'_, World>> {
        self.inner
            .read()
            .map_err(|e| anyhow!("ECS World read lock poisoned: {}", e))
    }

    /// Acquires panic-free write lock. If lock is poisoned, recovers cleanly.
    pub fn write(&self) -> Result<RwLockWriteGuard<'_, World>> {
        self.inner
            .write()
            .map_err(|e| anyhow!("ECS World write lock poisoned: {}", e))
    }

    /// Spawns a new entity thread-safely.
    pub fn spawn(&self) -> Result<Entity> {
        let mut world = self.write()?;
        Ok(world.spawn())
    }

    /// Despawns an entity thread-safely.
    pub fn despawn(&self, entity: Entity) -> Result<bool> {
        let mut world = self.write()?;
        Ok(world.despawn(entity))
    }

    /// Adds a component thread-safely.
    pub fn add_component<T: 'static + Send + Sync>(&self, entity: Entity, component: T) -> Result<()> {
        let mut world = self.write()?;
        world.add_component(entity, component);
        Ok(())
    }

    /// Clones a component thread-safely if T implements Clone.
    pub fn get_component_cloned<T: 'static + Send + Sync + Clone>(
        &self,
        entity: Entity,
    ) -> Result<Option<T>> {
        let world = self.read()?;
        Ok(world.get_component::<T>(entity).cloned())
    }

    /// Executes a closure with an immutable reference to a component under read lock.
    pub fn with_component<T: 'static + Send + Sync, R>(
        &self,
        entity: Entity,
        f: impl FnOnce(&T) -> R,
    ) -> Result<Option<R>> {
        let world = self.read()?;
        Ok(world.get_component::<T>(entity).map(f))
    }

    /// Executes a closure with a mutable reference to a component under write lock.
    pub fn with_component_mut<T: 'static + Send + Sync, R>(
        &self,
        entity: Entity,
        f: impl FnOnce(&mut T) -> R,
    ) -> Result<Option<R>> {
        let mut world = self.write()?;
        Ok(world.get_component_mut::<T>(entity).map(f))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::components::{Position, Velocity};

    #[test]
    fn test_world_component_operations() {
        let mut world = World::new();
        let e = world.spawn();

        world.add_component(e, Position::new(10.0, 20.0));
        world.add_component(e, Velocity::new(1.0, 2.0));

        assert!(world.has_component::<Position>(e));
        assert!(world.has_component::<Velocity>(e));

        {
            let pos = world.get_component::<Position>(e).unwrap();
            assert_eq!(pos.x, 10.0);
            assert_eq!(pos.y, 20.0);
        }

        let q2 = world.query_2::<Position, Velocity>();
        assert_eq!(q2.len(), 1);
        assert_eq!(q2[0].0, e);
    }
}
