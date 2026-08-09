use serde::{Deserialize, Serialize};
use std::fmt;

/// Generational Entity Index representing a unique entity in the ECS world.
/// The `generation` counter prevents Use-After-Free bugs when entity IDs are recycled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

impl Entity {
    pub const NULL: Entity = Entity {
        id: u32::MAX,
        generation: u32::MAX,
    };

    #[inline]
    pub fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        *self == Self::NULL
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {
            write!(f, "Entity(NULL)")
        } else {
            write!(f, "Entity({}:v{})", self.id, self.generation)
        }
    }
}

/// Thread-safe generational entity allocator.
#[derive(Debug, Default)]
pub struct EntityAllocator {
    generations: Vec<u32>,
    free_indices: Vec<u32>,
    active_count: usize,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            generations: Vec::with_capacity(1024),
            free_indices: Vec::with_capacity(256),
            active_count: 0,
        }
    }

    /// Allocates a new Entity slot or reuses a recycled index with an incremented generation.
    pub fn allocate(&mut self) -> Entity {
        self.active_count += 1;
        if let Some(id) = self.free_indices.pop() {
            let gen = self.generations[id as usize];
            Entity::new(id, gen)
        } else {
            let id = self.generations.len() as u32;
            self.generations.push(0);
            Entity::new(id, 0)
        }
    }

    /// Deallocates an Entity slot and increments its generation index.
    pub fn deallocate(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }

        let idx = entity.id as usize;
        self.generations[idx] = self.generations[idx].wrapping_add(1);
        self.free_indices.push(entity.id);
        self.active_count = self.active_count.saturating_sub(1);
        true
    }

    /// Validates whether an entity key is currently active and matching the current generation.
    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        if entity.is_null() {
            return false;
        }
        let idx = entity.id as usize;
        idx < self.generations.len() && self.generations[idx] == entity.generation
    }

    #[inline]
    pub fn get_generation(&self, id: u32) -> u32 {
        let idx = id as usize;
        if idx < self.generations.len() {
            self.generations[idx]
        } else {
            0
        }
    }

    #[inline]
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    #[inline]
    pub fn total_capacity(&self) -> usize {
        self.generations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_lifecycle() {
        let mut alloc = EntityAllocator::new();
        let e1 = alloc.allocate();
        let e2 = alloc.allocate();

        assert_eq!(e1.id, 0);
        assert_eq!(e1.generation, 0);
        assert_eq!(e2.id, 1);
        assert_eq!(e2.generation, 0);
        assert!(alloc.is_alive(e1));
        assert!(alloc.is_alive(e2));

        assert!(alloc.deallocate(e1));
        assert!(!alloc.is_alive(e1));

        let e3 = alloc.allocate();
        assert_eq!(e3.id, 0);
        assert_eq!(e3.generation, 1);
        assert!(alloc.is_alive(e3));
    }
}
