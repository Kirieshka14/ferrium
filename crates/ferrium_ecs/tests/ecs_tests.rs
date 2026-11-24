use ferrium_ecs::{Entity, Schedule, SparseSet, Stage, World};

#[derive(Debug, Clone, PartialEq)]
struct Position(f32, f32);

#[derive(Debug, Clone, PartialEq)]
struct Velocity(f32, f32);

#[derive(Debug, Clone, PartialEq)]
struct Health(i32);

#[derive(Debug, Default, PartialEq)]
struct StageLog(Vec<&'static str>);

#[test]
fn test_empty_world_state() {
    let world = World::new();
    assert_eq!(world.entity_count(), 0);
    assert!(!world.is_alive(Entity { id: 0, generation: 0 }));
}

#[test]
fn test_double_despawn_is_idempotent() {
    let mut world = World::new();
    let e = world.spawn();
    assert!(world.despawn(e));
    assert!(!world.despawn(e));
    assert_eq!(world.entity_count(), 0);
}

#[test]
fn test_insert_on_dead_entity_ignored() {
    let mut world = World::new();
    let e = world.spawn();
    world.despawn(e);
    world.insert(e, Position(1.0, 2.0));
    assert!(world.get::<Position>(e).is_none());
}

#[test]
fn test_component_overwrite() {
    let mut world = World::new();
    let e = world.spawn();
    world.insert(e, Health(100));
    world.insert(e, Health(45));
    assert_eq!(world.get::<Health>(e), Some(&Health(45)));
}

#[test]
fn test_component_get_mut() {
    let mut world = World::new();
    let e = world.spawn();
    world.insert(e, Health(50));
    world.get_mut::<Health>(e).unwrap().0 += 25;
    assert_eq!(world.get::<Health>(e), Some(&Health(75)));
}

#[test]
fn test_remove_single_component() {
    let mut world = World::new();
    let e = world.spawn();
    world.insert(e, Position(3.0, 4.0));
    world.insert(e, Velocity(1.0, -1.0));
    assert_eq!(world.remove::<Velocity>(e), Some(Velocity(1.0, -1.0)));
    assert!(world.get::<Velocity>(e).is_none());
    assert_eq!(world.get::<Position>(e), Some(&Position(3.0, 4.0)));
}

#[test]
fn test_despawn_cleans_all_pools() {
    let mut world = World::new();
    let e = world.spawn();
    world.insert(e, Position(1.0, 1.0));
    world.insert(e, Velocity(2.0, 2.0));
    world.insert(e, Health(99));
    world.despawn(e);
    assert_eq!(world.storage::<Position>().map(|s| s.len()), Some(0));
    assert_eq!(world.storage::<Velocity>().map(|s| s.len()), Some(0));
    assert_eq!(world.storage::<Health>().map(|s| s.len()), Some(0));
}

#[test]
fn test_stale_generation_isolation() {
    let mut world = World::new();
    let old = world.spawn();
    world.insert(old, Health(10));
    world.despawn(old);

    let fresh = world.spawn();
    assert_eq!(fresh.id, old.id);
    assert_ne!(fresh.generation, old.generation);
    world.insert(fresh, Health(999));
    assert!(world.get::<Health>(old).is_none());
    assert_eq!(world.get::<Health>(fresh), Some(&Health(999)));
}

#[test]
fn test_sparse_set_iteration() {
    let mut set = SparseSet::new();
    let entities: Vec<Entity> = (0..5).map(|id| Entity { id, generation: 0 }).collect();
    for (i, &e) in entities.iter().enumerate() {
        set.insert(e, i as i32 * 10);
    }
    set.remove(entities[2]);
    let sum: i32 = set.iter().map(|(_, v)| *v).sum();
    assert_eq!(set.len(), 4);
    assert_eq!(sum, 0 + 10 + 30 + 40);
}

#[test]
fn test_sparse_set_remove_last() {
    let mut set = SparseSet::new();
    let e0 = Entity { id: 0, generation: 0 };
    let e1 = Entity { id: 1, generation: 0 };
    set.insert(e0, "a");
    set.insert(e1, "b");
    assert_eq!(set.remove(e1), Some("b"));
    assert_eq!(set.get(e0), Some(&"a"));
    assert_eq!(set.len(), 1);
}
