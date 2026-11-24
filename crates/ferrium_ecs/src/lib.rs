pub mod entity;
pub mod sparse_set;
pub mod world;

pub use entity::Entity;
pub use sparse_set::{ComponentPool, SparseSet};
pub use world::{Schedule, Stage, World};

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Pos(f32, f32);

    #[derive(Debug, PartialEq)]
    struct TickCounter(u32);

    #[test]
    fn test_spawn_despawn_and_generation_reuse() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        assert_eq!(world.entity_count(), 2);

        world.insert(e1, Pos(1.0, 2.0));
        world.insert(e2, Pos(3.0, 4.0));
        assert!(world.despawn(e1));
        assert!(!world.is_alive(e1));
        assert!(world.get::<Pos>(e1).is_none());
        assert_eq!(world.get::<Pos>(e2), Some(&Pos(3.0, 4.0)));

        let e3 = world.spawn();
        assert_eq!(e3.id, e1.id);
        assert_eq!(e3.generation, 1);
        assert!(world.is_alive(e3));
        assert!(!world.is_alive(e1));
    }

    #[test]
    fn test_sparse_set_swap_remove_consistency() {
        let mut set = SparseSet::new();
        let e0 = Entity { id: 0, generation: 0 };
        let e1 = Entity { id: 1, generation: 0 };
        let e2 = Entity { id: 2, generation: 0 };

        set.insert(e0, 10);
        set.insert(e1, 20);
        set.insert(e2, 30);

        assert_eq!(set.remove(e0), Some(10));
        assert_eq!(set.get(e0), None);
        assert_eq!(set.get(e1), Some(&20));
        assert_eq!(set.get(e2), Some(&30));
    }

    #[test]
    fn test_resources_and_schedule_order() {
        let mut world = World::new();
        world.insert_resource(TickCounter(0));

        let mut schedule = Schedule::new();
        schedule.add_system(Stage::Combat, |w| {
            let c = w.resource_mut::<TickCounter>().unwrap();
            c.0 *= 10;
        });
        schedule.add_system(Stage::Input, |w| {
            let c = w.resource_mut::<TickCounter>().unwrap();
            c.0 += 3;
        });

        schedule.run_all(&mut world);
        // Input runs before Combat: (0 + 3) * 10 = 30
        assert_eq!(world.resource::<TickCounter>(), Some(&TickCounter(30)));
    }
}
