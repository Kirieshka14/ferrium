use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::entity::Entity;
use crate::sparse_set::{ComponentPool, SparseSet};

/// Central ECS registry storing entities, sparse-set component pools, and singleton resources.
pub struct World {
    generations: Vec<u32>,
    alive: Vec<bool>,
    free_ids: Vec<u32>,
    alive_count: usize,
    pools: HashMap<TypeId, Box<dyn ComponentPool>>,
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            alive: Vec::new(),
            free_ids: Vec::new(),
            alive_count: 0,
            pools: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        self.alive_count += 1;
        if let Some(id) = self.free_ids.pop() {
            let idx = id as usize;
            self.alive[idx] = true;
            Entity {
                id,
                generation: self.generations[idx],
            }
        } else {
            let id = self.generations.len() as u32;
            self.generations.push(0);
            self.alive.push(true);
            Entity { id, generation: 0 }
        }
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }
        let idx = entity.id as usize;
        self.alive[idx] = false;
        self.generations[idx] = self.generations[idx].wrapping_add(1);
        self.free_ids.push(entity.id);
        self.alive_count -= 1;

        for pool in self.pools.values_mut() {
            pool.remove_entity(entity);
        }
        true
    }

    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        let idx = entity.id as usize;
        idx < self.alive.len() && self.alive[idx] && self.generations[idx] == entity.generation
    }

    #[inline]
    pub fn entity_count(&self) -> usize {
        self.alive_count
    }

    pub fn insert<T: 'static>(&mut self, entity: Entity, component: T) {
        if !self.is_alive(entity) {
            return;
        }
        let type_id = TypeId::of::<T>();
        let pool = self
            .pools
            .entry(type_id)
            .or_insert_with(|| Box::new(SparseSet::<T>::new()));
        let set = pool
            .as_any_mut()
            .downcast_mut::<SparseSet<T>>()
            .expect("Component pool type mismatch");
        set.insert(entity, component);
    }

    pub fn remove<T: 'static>(&mut self, entity: Entity) -> Option<T> {
        let pool = self.pools.get_mut(&TypeId::of::<T>())?;
        pool.as_any_mut()
            .downcast_mut::<SparseSet<T>>()?
            .remove(entity)
    }

    pub fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        if !self.is_alive(entity) {
            return None;
        }
        self.storage::<T>()?.get(entity)
    }

    pub fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.is_alive(entity) {
            return None;
        }
        self.storage_mut::<T>()?.get_mut(entity)
    }

    pub fn storage<T: 'static>(&self) -> Option<&SparseSet<T>> {
        self.pools
            .get(&TypeId::of::<T>())?
            .as_any()
            .downcast_ref::<SparseSet<T>>()
    }

    pub fn storage_mut<T: 'static>(&mut self) -> Option<&mut SparseSet<T>> {
        self.pools
            .get_mut(&TypeId::of::<T>())?
            .as_any_mut()
            .downcast_mut::<SparseSet<T>>()
    }

    #[inline]
    pub fn entities_slice<T: 'static>(&self) -> &[Entity] {
        self.storage::<T>().map(|s| s.entities()).unwrap_or(&[])
    }

    pub fn entities_with<T: 'static>(&self) -> Vec<Entity> {
        self.entities_slice::<T>().to_vec()
    }

    pub fn insert_resource<R: 'static>(&mut self, resource: R) {
        self.resources.insert(TypeId::of::<R>(), Box::new(resource));
    }

    pub fn resource<R: 'static>(&self) -> Option<&R> {
        self.resources.get(&TypeId::of::<R>())?.downcast_ref::<R>()
    }

    pub fn resource_mut<R: 'static>(&mut self) -> Option<&mut R> {
        self.resources.get_mut(&TypeId::of::<R>())?.downcast_mut::<R>()
    }
}

/// Ordered execution stages for deterministic ECS system scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    Input = 0,
    Script = 1,
    FixedUpdate = 2,
    Combat = 3,
    Cleanup = 4,
}

pub type SystemFn = Box<dyn FnMut(&mut World)>;

#[derive(Default)]
pub struct Schedule {
    stages: [Vec<SystemFn>; 5],
}

impl Schedule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_system<F>(&mut self, stage: Stage, sys: F)
    where
        F: FnMut(&mut World) + 'static,
    {
        self.stages[stage as usize].push(Box::new(sys));
    }

    pub fn run_all(&mut self, world: &mut World) {
        for stage_systems in &mut self.stages {
            for sys in stage_systems {
                sys(world);
            }
        }
    }
}
