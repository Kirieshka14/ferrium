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
