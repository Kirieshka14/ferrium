use std::any::Any;

use crate::entity::Entity;

const EMPTY_SLOT: u32 = u32::MAX;

/// Type-erased interface for sparse-set component pools stored inside `World`.
pub trait ComponentPool: Any {
    fn remove_entity(&mut self, entity: Entity);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Cache-friendly dense packed array indexed by compact 32-bit sparse entity IDs.
pub struct SparseSet<T: 'static> {
    sparse: Vec<u32>,
    dense_entities: Vec<Entity>,
    dense_data: Vec<T>,
}

impl<T: 'static> Default for SparseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> SparseSet<T> {
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense_entities: Vec::new(),
            dense_data: Vec::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity, value: T) {
        let id = entity.id as usize;
        if id >= self.sparse.len() {
            self.sparse.resize(id + 1, EMPTY_SLOT);
        }

        let slot = self.sparse[id];
        if slot != EMPTY_SLOT {
            let dense_idx = slot as usize;
            self.dense_entities[dense_idx] = entity;
            self.dense_data[dense_idx] = value;
            return;
        }

        let dense_idx = self.dense_data.len();
        debug_assert!(dense_idx < EMPTY_SLOT as usize);
        self.sparse[id] = dense_idx as u32;
        self.dense_entities.push(entity);
        self.dense_data.push(value);
    }

    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let id = entity.id as usize;
        let slot = *self.sparse.get(id)?;
        if slot == EMPTY_SLOT {
            return None;
        }
        let dense_idx = slot as usize;
        if self.dense_entities[dense_idx] != entity {
            return None;
        }

        self.sparse[id] = EMPTY_SLOT;
        let last_idx = self.dense_data.len() - 1;
        let removed_val = self.dense_data.swap_remove(dense_idx);
        self.dense_entities.swap_remove(dense_idx);

        if dense_idx != last_idx {
            let moved_entity = self.dense_entities[dense_idx];
            self.sparse[moved_entity.id as usize] = dense_idx as u32;
        }

        Some(removed_val)
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<&T> {
        let slot = *self.sparse.get(entity.id as usize)?;
        if slot == EMPTY_SLOT {
            return None;
        }
        let dense_idx = slot as usize;
        if self.dense_entities[dense_idx] == entity {
            Some(&self.dense_data[dense_idx])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let slot = *self.sparse.get(entity.id as usize)?;
        if slot == EMPTY_SLOT {
            return None;
        }
        let dense_idx = slot as usize;
        if self.dense_entities[dense_idx] == entity {
            Some(&mut self.dense_data[dense_idx])
        } else {
            None
        }
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.get(entity).is_some()
    }

    #[inline]
    pub fn entities(&self) -> &[Entity] {
        &self.dense_entities
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.dense_entities
            .iter()
            .copied()
            .zip(self.dense_data.iter())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        self.dense_entities
            .iter()
            .copied()
            .zip(self.dense_data.iter_mut())
    }
}

impl<T: 'static> ComponentPool for SparseSet<T> {
    fn remove_entity(&mut self, entity: Entity) {
        self.remove(entity);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn len(&self) -> usize {
        self.dense_data.len()
    }
}
