use ferrium_ecs::Entity;
use ferrium_math::{Color, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    Player,
    Boss,
}

#[derive(Debug, Clone)]
pub struct Health {
    pub current: i32,
    pub max: i32,
    pub invuln_timer: f32,
    pub flash_timer: f32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self {
            current: max,
            max,
            invuln_timer: 0.0,
            flash_timer: 0.0,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Collider {
    pub half_extents: Vec2,
}

#[derive(Debug, Clone)]
pub struct Hitbox {
    pub owner: Entity,
    pub team: Team,
    pub damage: i32,
    pub knockback: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub hit_entities: Vec<Entity>,
    pub follow_owner: bool,
    pub offset: Vec2,
    pub size: Vec2,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub speed: f32,
    pub dash_timer: f32,
    pub dash_cooldown: f32,
    pub dash_dir: Vec2,
    pub attack_cooldown: f32,
    pub facing: Vec2,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 270.0,
            dash_timer: 0.0,
            dash_cooldown: 0.0,
            dash_dir: Vec2::new(1.0, 0.0),
            attack_cooldown: 0.0,
            facing: Vec2::new(1.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Boss {
    pub state: String,
    pub phase: u8,
    pub telegraph_pulse: f32,
}

impl Default for Boss {
    fn default() -> Self {
        Self {
            state: "boot".into(),
            phase: 1,
            telegraph_pulse: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub vel: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub size: f32,
    pub color: Color,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_is_dead_threshold() {
        let mut hp = Health::new(100);
        assert!(!hp.is_dead());
        hp.current = 0;
        assert!(hp.is_dead());
        hp.current = -15;
        assert!(hp.is_dead());
    }

    #[test]
    fn test_player_and_boss_defaults() {
        let player = Player::default();
        assert!(player.speed > 0.0);
        assert_eq!(player.facing, Vec2::new(1.0, 0.0));

        let boss = Boss::default();
        assert_eq!(boss.phase, 1);
        assert_eq!(boss.state, "boot");
    }
}
