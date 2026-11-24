use std::collections::VecDeque;
use std::path::PathBuf;

use ferrium_ecs::{Entity, World};
use ferrium_math::{Color, Rect, Transform, Vec2, Vec3};
use ferrium_render::{Camera, ProjectionMode, RenderBackend, RenderQueue, SoftwareCanvas};
use ferrium_script::{
    FerriumHost, JitPolicy, JitTelemetry, ScriptCommand, ScriptComponent, ScriptSnapshot, VmResult,
};

use crate::components::{Boss, Collider, Health, Hitbox, Particle, Player, Team, Velocity};
use crate::input::InputState;

const FALLBACK_BOSS_SCRIPT: &str = r#"
purr("Fallback Boss AI online (FustyLang 0.2.1)");
while true {
    let dir = dir_to_player();
    set_boss_state("chase");
    set_self_vel(dir * 160.0);
    wait 0.25;
}
"#;

pub struct FerriumGame {
    pub world: World,
    pub camera: Camera,
    pub queue: RenderQueue,
    pub player_entity: Entity,
    pub boss_entity: Entity,
    pub logs: VecDeque<String>,
    pub hitstop_timer: f32,
    pub show_debug_hitboxes: bool,
    pub arena_half_size: Vec2,
    pub script_path: PathBuf,
    pub jit_policy: JitPolicy,
    pub elapsed: f64,
    pub tick_count: u64,
    pub fps: f32,
    pub frame_arena_resets: u64,
    pub total_hitboxes_spawned: u64,
    pub saw_meow_teleport: bool,
}

impl FerriumGame {
    #[allow(dead_code)]
    pub fn new(script_path: PathBuf) -> Self {
        Self::with_jit_policy(script_path, JitPolicy::default())
    }

    pub fn with_jit_policy(script_path: PathBuf, jit_policy: JitPolicy) -> Self {
        let mut game = Self {
            world: World::new(),
            camera: Camera::new(),
            queue: RenderQueue::new(),
            player_entity: Entity { id: 0, generation: 0 },
            boss_entity: Entity { id: 0, generation: 0 },
            logs: VecDeque::with_capacity(8),
            hitstop_timer: 0.0,
            show_debug_hitboxes: true,
            arena_half_size: Vec2::new(420.0, 250.0),
            script_path,
            jit_policy,
            elapsed: 0.0,
            tick_count: 0,
            fps: 60.0,
            frame_arena_resets: 0,
            total_hitboxes_spawned: 0,
            saw_meow_teleport: false,
        };
        game.populate_arena();
        game
    }

    pub fn set_jit_policy(&mut self, policy: JitPolicy) {
        self.jit_policy = policy;
        for entity in self.world.entities_with::<ScriptComponent>() {
            if let Some(script) = self.world.get_mut::<ScriptComponent>(entity) {
                script.set_jit_policy(policy);
            }
        }
    }

    pub fn jit_telemetry(&self) -> JitTelemetry {
        let mut telemetry = JitTelemetry::default();
        for entity in self.world.entities_with::<ScriptComponent>() {
            if let Some(script) = self.world.get::<ScriptComponent>(entity) {
                telemetry.accumulate(script.vm.jit_stats());
            }
        }
        telemetry
    }

    pub fn reset(&mut self) {
        let mode = self.camera.mode;
        let show_debug = self.show_debug_hitboxes;
        self.world = World::new();
        self.camera = Camera::new();
        self.camera.mode = mode;
        self.show_debug_hitboxes = show_debug;
        self.logs.clear();
        self.hitstop_timer = 0.0;
        self.elapsed = 0.0;
        self.populate_arena();
        self.push_log("Arena reset");
    }

    fn populate_arena(&mut self) {
        let player = self.world.spawn();
        self.world.insert(player, Transform::from_xyz(-200.0, 0.0, 2.0));
        self.world.insert(player, Velocity(Vec2::ZERO));
        self.world.insert(player, Health::new(100));
        self.world.insert(
            player,
            Collider {
                half_extents: Vec2::new(16.0, 16.0),
            },
        );
        self.world.insert(player, Team::Player);
        self.world.insert(player, Player::default());
        self.player_entity = player;

        let boss = self.world.spawn();
        self.world.insert(boss, Transform::from_xyz(180.0, 0.0, 2.0));
        self.world.insert(boss, Velocity(Vec2::ZERO));
        self.world.insert(boss, Health::new(1000));
        self.world.insert(
            boss,
            Collider {
                half_extents: Vec2::new(24.0, 24.0),
            },
        );
        self.world.insert(boss, Team::Boss);
        self.world.insert(boss, Boss::default());

        let script = match ScriptComponent::from_file_with_jit(&self.script_path, self.jit_policy) {
            Ok(s) => {
                self.push_log(format!(
                    "Loaded {} ({})",
                    self.script_path.display(),
                    self.jit_policy.label()
                ));
                s
            }
            Err(err) => {
                self.push_log(format!("Fallback script ({})", err));
                let mut s = ScriptComponent::from_source_with_jit(
                    &self.script_path,
                    FALLBACK_BOSS_SCRIPT,
                    self.jit_policy,
                )
                .expect("Fallback Fusty script must compile");
                s.last_error = Some(err);
                s
            }
        };
        self.world.insert(boss, script);
        self.boss_entity = boss;
    }

    pub fn push_log(&mut self, msg: impl Into<String>) {
        if self.logs.len() >= 5 {
            self.logs.pop_front();
        }
        self.logs.push_back(msg.into());
    }

    pub fn spawn_particle_burst(&mut self, pos: Vec2, count: u32, color: Color, speed: f32) {
        for i in 0..count {
            let angle = (i as f32 / count.max(1) as f32) * std::f32::consts::TAU
                + (self.elapsed as f32 * 3.7);
            let mag = speed * (0.45 + ((i * 37) % 10) as f32 * 0.07);
            let p = self.world.spawn();
            self.world.insert(p, Transform::from_xyz(pos.x, pos.y, 4.0));
            self.world.insert(
                p,
                Particle {
                    vel: Vec2::from_angle(angle) * mag,
                    lifetime: 0.32 + ((i % 4) as f32) * 0.06,
                    max_lifetime: 0.50,
                    size: 4.5 + (i % 3) as f32 * 1.5,
                    color,
                },
            );
        }
    }

    pub fn tick(&mut self, dt: f32, input: &InputState) {
        let dt = dt.clamp(0.001, 0.05);
        self.elapsed += dt as f64;
        if dt > 0.0 {
            let inst_fps = 1.0 / dt;
            self.fps = self.fps * 0.9 + inst_fps * 0.1;
        }

        if input.toggle_debug_pressed {
            self.show_debug_hitboxes = !self.show_debug_hitboxes;
        }
        if input.toggle_3d_pressed {
            self.camera.mode = match self.camera.mode {
                ProjectionMode::Hybrid25D => ProjectionMode::Orthographic2D,
                ProjectionMode::Orthographic2D => ProjectionMode::Hybrid25D,
            };
        }
        if input.reset_pressed {
            self.reset();
            return;
        }

        if input.force_reload_pressed {
            if let Some(script) = self.world.get_mut::<ScriptComponent>(self.boss_entity) {
                match script.try_hot_reload(true) {
                    Some(Ok(n)) => self.push_log(format!("Hot-reloaded #{} (F5)", n)),
                    Some(Err(e)) => self.push_log(format!("Reload error: {}", e)),
                    None => {}
                }
            }
        }

        self.camera.update(dt, self.elapsed as f32);

        if self.hitstop_timer > 0.0 {
            self.hitstop_timer = (self.hitstop_timer - dt).max(0.0);
            self.update_particles(dt * 0.35);
            return;
        }

        self.update_player(dt, input);
        self.update_scripts(dt);
        self.integrate_physics(dt);
        self.resolve_combat();
        self.update_particles(dt);
        self.update_camera(dt);
    }

    fn update_player(&mut self, dt: f32, input: &InputState) {
        let player_dead = self
            .world
            .get::<Health>(self.player_entity)
            .map(|h| h.is_dead())
            .unwrap_or(true);
        if player_dead {
            if let Some(vel) = self.world.get_mut::<Velocity>(self.player_entity) {
                vel.0 = Vec2::ZERO;
            }
            return;
        }

        let pos = self
            .world
            .get::<Transform>(self.player_entity)
            .map(|t| t.pos2())
            .unwrap_or(Vec2::ZERO);

        let move_dir = input.move_axis();
        let mut spawn_slash: Option<(Vec2, Vec2)> = None;
        let mut dash_burst = false;

        if let Some(player) = self.world.get_mut::<Player>(self.player_entity) {
            player.dash_cooldown = (player.dash_cooldown - dt).max(0.0);
            player.attack_cooldown = (player.attack_cooldown - dt).max(0.0);

            if let Some(mouse_w) = input.mouse_world {
                let to_mouse = mouse_w - pos;
                if to_mouse.length_sq() > 9.0 {
                    player.facing = to_mouse.normalized();
                }
            } else if move_dir.length_sq() > 1e-4 {
                player.facing = move_dir;
            }

            if input.dash_just_pressed && player.dash_cooldown <= 0.0 {
                player.dash_timer = 0.16;
                player.dash_cooldown = 0.65;
                player.dash_dir = if move_dir.length_sq() > 1e-4 {
                    move_dir
                } else {
                    player.facing
                };
                dash_burst = true;
            }

            let target_vel = if player.dash_timer > 0.0 {
                player.dash_timer = (player.dash_timer - dt).max(0.0);
                player.dash_dir * 690.0
            } else {
                move_dir * player.speed
            };

            if input.attack_just_pressed && player.attack_cooldown <= 0.0 {
                player.attack_cooldown = 0.24;
                spawn_slash = Some((pos, player.facing));
            }

            if let Some(vel) = self.world.get_mut::<Velocity>(self.player_entity) {
                vel.0 = target_vel;
            }
        }

        if dash_burst {
            if let Some(hp) = self.world.get_mut::<Health>(self.player_entity) {
                hp.invuln_timer = hp.invuln_timer.max(0.18);
            }
            self.spawn_particle_burst(pos, 8, Color::CYAN, 170.0);
        }

        if let Some((origin, facing)) = spawn_slash {
            let hb = self.world.spawn();
            let offset = facing * 34.0;
            self.world
                .insert(hb, Transform::from_xyz(origin.x + offset.x, origin.y + offset.y, 3.0));
            self.world.insert(
                hb,
                Hitbox {
                    owner: self.player_entity,
                    team: Team::Player,
                    damage: 45,
                    knockback: 220.0,
                    lifetime: 0.14,
                    max_lifetime: 0.14,
                    hit_entities: Vec::new(),
                    follow_owner: true,
                    offset,
                    size: Vec2::new(62.0, 62.0),
                },
            );
            self.total_hitboxes_spawned += 1;
            self.spawn_particle_burst(origin + offset, 6, Color::CYAN, 150.0);
        }
    }

    fn update_scripts(&mut self, dt: f32) {
        let player_pos = self
            .world
            .get::<Transform>(self.player_entity)
            .map(|t| t.pos2())
            .unwrap_or(Vec2::ZERO);
        let player_hp = self
            .world
            .get::<Health>(self.player_entity)
            .map(|h| h.current)
            .unwrap_or(0);

        let scripted = self.world.entities_with::<ScriptComponent>();
        let mut deferred_commands = Vec::new();
        let mut reload_events = Vec::new();
        let mut runtime_errors = Vec::new();

        for entity in scripted {
            let (self_pos, self_pos3, self_rot, self_scale) = self
                .world
                .get::<Transform>(entity)
                .map(|t| (t.pos2(), t.position, t.rotation, t.scale))
                .unwrap_or((Vec2::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ONE));
            let self_vel = self
                .world
                .get::<Velocity>(entity)
                .map(|v| v.0)
                .unwrap_or(Vec2::ZERO);
            let (self_hp, self_max_hp) = self
                .world
                .get::<Health>(entity)
                .map(|h| (h.current, h.max))
                .unwrap_or((0, 1));

            let mut used_meow = false;
            if let Some(script) = self.world.get_mut::<ScriptComponent>(entity) {
                script.meow_flash_timer = (script.meow_flash_timer - dt).max(0.0);
                script.poll_timer += dt as f64;
                if script.poll_timer >= 0.25 {
                    script.poll_timer = 0.0;
                    if let Some(res) = script.try_hot_reload(false) {
                        reload_events.push(res);
                    }
                }

                if self_hp > 0 {
                    if script.status == "done" {
                        script.vm.reset_frame();
                        self.frame_arena_resets += 1;
                        continue;
                    }
                    script.wait_remaining = (script.wait_remaining - dt as f64).max(0.0);
                    if script.wait_remaining <= 0.0 {
                        let snapshot = ScriptSnapshot {
                            self_entity: entity,
                            self_pos,
                            self_pos3,
                            self_rot,
                            self_scale,
                            self_vel,
                            self_hp,
                            self_max_hp,
                            player_pos,
                            player_hp,
                            input_x: 0.0,
                            input_y: 0.0,
                            dt: dt as f64,
                            elapsed_time: self.elapsed,
                            frame_count: self.tick_count,
                            jit_enabled: script.jit_policy.is_enabled(),
                            jit_compiled_count: script.vm.jit_stats().compiled_functions,
                        };
                        let mut host = FerriumHost::new(snapshot);
                        match script.step(&mut host) {
                            Ok(VmResult::Waiting { duration_sec }) => {
                                script.status = format!("wait {:.2}s", duration_sec);
                            }
                            Ok(VmResult::Yielded) => {
                                script.status = "yield".into();
                            }
                            Ok(VmResult::Done(_)) => {
                                script.status = "done".into();
                            }
                            Err(err) => {
                                runtime_errors.push(err.to_string());
                            }
                        }
                        used_meow = host.used_meow;
                        deferred_commands.extend(host.commands);
                    }
                } else {
                    script.status = "defeated".into();
                }

                // O(1) Frame Arena pointer reset at the end of every tick!
                script.vm.reset_frame();
                self.frame_arena_resets += 1;
                self.tick_count += 1;

                if used_meow {
                    script.meow_flash_timer = 0.75;
                    self.saw_meow_teleport = true;
                }
            }
        }

        for ev in reload_events {
            match ev {
                Ok(n) => self.push_log(format!("Hot-reloaded script #{}", n)),
                Err(e) => self.push_log(format!("Reload err: {}", e)),
            }
        }
        for err in runtime_errors {
            self.push_log(err);
        }

        for cmd in deferred_commands {
            match cmd {
                ScriptCommand::SetPos(e, pos) => {
                    if let Some(t) = self.world.get_mut::<Transform>(e) {
                        t.set_pos2(pos);
                    }
                }
                ScriptCommand::SetPos3(e, pos) => {
                    if let Some(t) = self.world.get_mut::<Transform>(e) {
                        t.position = pos;
                    }
                }
                ScriptCommand::SetRot(e, rot) => {
                    if let Some(t) = self.world.get_mut::<Transform>(e) {
                        t.rotation = rot;
                    }
                }
                ScriptCommand::SetScale(e, scale) => {
                    if let Some(t) = self.world.get_mut::<Transform>(e) {
                        t.scale = scale;
                    }
                }
                ScriptCommand::SetColor(_, _) => {}
                ScriptCommand::SetVel(e, vel) => {
                    if let Some(v) = self.world.get_mut::<Velocity>(e) {
                        v.0 = vel;
                    }
                }
                ScriptCommand::RawTeleport(e, pos) => {
                    let old_pos = self
                        .world
                        .get::<Transform>(e)
                        .map(|t| t.pos2())
                        .unwrap_or(pos);
                    let clamped = Vec2::new(
                        pos.x.clamp(-self.arena_half_size.x + 28.0, self.arena_half_size.x - 28.0),
                        pos.y.clamp(-self.arena_half_size.y + 28.0, self.arena_half_size.y - 28.0),
                    );
                    if let Some(t) = self.world.get_mut::<Transform>(e) {
                        t.set_pos2(clamped);
                    }
                    self.spawn_particle_burst(old_pos, 10, Color::MAGENTA, 190.0);
                }
                ScriptCommand::RawTeleport3(e, pos) => {
                    let old_pos = self
                        .world
                        .get::<Transform>(e)
                        .map(|t| t.pos2())
                        .unwrap_or(pos.xy());
                    if let Some(t) = self.world.get_mut::<Transform>(e) {
                        t.position = pos;
                    }
                    self.spawn_particle_burst(old_pos, 10, Color::MAGENTA, 190.0);
                }
                ScriptCommand::SpawnHitbox {
                    owner,
                    offset,
                    size,
                    damage,
                    duration,
                    knockback,
                } => {
                    let owner_pos = self
                        .world
                        .get::<Transform>(owner)
                        .map(|t| t.pos2())
                        .unwrap_or(Vec2::ZERO);
                    let hb = self.world.spawn();
                    self.world.insert(
                        hb,
                        Transform::from_xyz(owner_pos.x + offset.x, owner_pos.y + offset.y, 3.0),
                    );
                    self.world.insert(
                        hb,
                        Hitbox {
                            owner,
                            team: Team::Boss,
                            damage,
                            knockback,
                            lifetime: duration,
                            max_lifetime: duration,
                            hit_entities: Vec::new(),
                            follow_owner: true,
                            offset,
                            size,
                        },
                    );
                    self.total_hitboxes_spawned += 1;
                }
                ScriptCommand::SetState { entity, state } => {
                    if let Some(boss) = self.world.get_mut::<Boss>(entity) {
                        boss.state = state;
                    }
                }
                ScriptCommand::CameraShake { intensity, duration } => {
                    self.camera.trigger_shake(intensity, duration);
                }
                ScriptCommand::SpawnParticles { pos, count, color } => {
                    self.spawn_particle_burst(pos, count, color, 180.0);
                }
                ScriptCommand::Heal { entity, amount } => {
                    if let Some(h) = self.world.get_mut::<Health>(entity) {
                        h.current = (h.current + amount).min(h.max);
                    }
                }
                ScriptCommand::Damage { entity, amount } => {
                    if let Some(h) = self.world.get_mut::<Health>(entity) {
                        h.current = (h.current - amount).max(0);
                    }
                }
                ScriptCommand::EmitEvent { name, payload, .. } => {
                    if payload.is_empty() {
                        self.push_log(format!("[EVENT] {}", name));
                    } else {
                        self.push_log(format!("[EVENT] {}: {}", name, payload));
                    }
                }
                ScriptCommand::Log(msg) => {
                    self.push_log(msg);
                }
                ScriptCommand::Purr(msg) => {
                    self.push_log(format!("[purr] {}", msg));
                }
            }
        }

        // Keep boss phase synchronized with current health
        let boss_hp = self
            .world
            .get::<Health>(self.boss_entity)
            .map(|h| (h.current, h.max));
        if let (Some((hp, max_hp)), Some(boss)) =
            (boss_hp, self.world.get_mut::<Boss>(self.boss_entity))
        {
            boss.telegraph_pulse += dt * 7.0;
            boss.phase = if hp <= max_hp / 2 { 2 } else { 1 };
            if hp <= 0 {
                boss.state = "defeated".into();
            }
        }
    }

    fn integrate_physics(&mut self, dt: f32) {
        let moving = self.world.entities_with::<Velocity>();
        for e in moving {
            let vel = self.world.get::<Velocity>(e).map(|v| v.0).unwrap_or(Vec2::ZERO);
            let pad = self
                .world
                .get::<Collider>(e)
                .map(|c| c.half_extents)
                .unwrap_or(Vec2::new(16.0, 16.0));
            if let Some(t) = self.world.get_mut::<Transform>(e) {
                let next = t.pos2() + vel * dt;
                let max_x = self.arena_half_size.x - pad.x;
                let max_y = self.arena_half_size.y - pad.y;
                t.set_pos2(Vec2::new(
                    next.x.clamp(-max_x, max_x),
                    next.y.clamp(-max_y, max_y),
                ));
            }
        }

        for e in self.world.entities_with::<Health>() {
            if let Some(hp) = self.world.get_mut::<Health>(e) {
                hp.invuln_timer = (hp.invuln_timer - dt).max(0.0);
                hp.flash_timer = (hp.flash_timer - dt).max(0.0);
            }
        }

        let mut expired_hitboxes = Vec::new();
        for hb_entity in self.world.entities_with::<Hitbox>() {
            let (owner, follow, offset, expired) = {
                let hb = self.world.get_mut::<Hitbox>(hb_entity).unwrap();
                hb.lifetime -= dt;
                (hb.owner, hb.follow_owner, hb.offset, hb.lifetime <= 0.0)
            };
            if expired {
                expired_hitboxes.push(hb_entity);
            } else if follow {
                if let Some(owner_pos) = self.world.get::<Transform>(owner).map(|t| t.pos2()) {
                    if let Some(hb_t) = self.world.get_mut::<Transform>(hb_entity) {
                        hb_t.set_pos2(owner_pos + offset);
                    }
                }
            }
        }
        for e in expired_hitboxes {
            self.world.despawn(e);
        }
    }

    fn resolve_combat(&mut self) {
        struct HitEvent {
            hitbox: Entity,
            target: Entity,
            damage: i32,
            knockback_dir: Vec2,
            knockback: f32,
            hit_pos: Vec2,
            is_boss_target: bool,
        }

        let mut hits = Vec::new();
        {
            let hitboxes = self.world.entities_slice::<Hitbox>();
            let targets = self.world.entities_slice::<Collider>();
            for &hb_entity in hitboxes {
                let Some(hb_pos) = self.world.get::<Transform>(hb_entity).map(|t| t.pos2()) else {
                    continue;
                };
                let Some(hb) = self.world.get::<Hitbox>(hb_entity) else {
                    continue;
                };
                let hb_rect = Rect::from_center_size(hb_pos, hb.size);

                for &target in targets {
                    if target == hb.owner || hb.hit_entities.contains(&target) {
                        continue;
                    }
                    let Some(&target_team) = self.world.get::<Team>(target) else {
                        continue;
                    };
                    if target_team == hb.team {
                        continue;
                    }
                    let Some(hp) = self.world.get::<Health>(target) else {
                        continue;
                    };
                    if hp.is_dead() || hp.invuln_timer > 0.0 {
                        continue;
                    }
                    let Some(t_pos) = self.world.get::<Transform>(target).map(|t| t.pos2()) else {
                        continue;
                    };
                    let Some(col) = self.world.get::<Collider>(target) else {
                        continue;
                    };
                    let t_rect = Rect::from_center_size(t_pos, col.half_extents * 2.0);
                    if hb_rect.intersects(&t_rect) {
                        let dir = (t_pos - hb_pos).normalized();
                        hits.push(HitEvent {
                            hitbox: hb_entity,
                            target,
                            damage: hb.damage,
                            knockback_dir: if dir.length_sq() > 1e-4 {
                                dir
                            } else {
                                Vec2::new(1.0, 0.0)
                            },
                            knockback: hb.knockback,
                            hit_pos: t_pos,
                            is_boss_target: target_team == Team::Boss,
                        });
                    }
                }
            }
        }

        for ev in hits {
            if let Some(hb) = self.world.get_mut::<Hitbox>(ev.hitbox) {
                hb.hit_entities.push(ev.target);
            }
            if let Some(hp) = self.world.get_mut::<Health>(ev.target) {
                hp.current = (hp.current - ev.damage).max(0);
                hp.invuln_timer = if ev.is_boss_target { 0.14 } else { 0.45 };
                hp.flash_timer = 0.14;
            }
            if let Some(vel) = self.world.get_mut::<Velocity>(ev.target) {
                vel.0 += ev.knockback_dir * ev.knockback;
            }
            self.hitstop_timer = 0.04;
            self.camera.trigger_shake(if ev.is_boss_target { 5.5 } else { 8.5 }, 0.15);
            let spark_color = if ev.is_boss_target {
                Color::AMBER
            } else {
                Color::CRIMSON
            };
            self.spawn_particle_burst(ev.hit_pos, 12, spark_color, 250.0);
        }
    }

    fn update_particles(&mut self, dt: f32) {
        let mut dead = Vec::new();
        for e in self.world.entities_with::<Particle>() {
            let (vel, expired) = {
                let p = self.world.get_mut::<Particle>(e).unwrap();
                p.lifetime -= dt;
                p.vel = p.vel * (1.0 - (dt * 4.5).min(0.9));
                (p.vel, p.lifetime <= 0.0)
            };
            if expired {
                dead.push(e);
            } else if let Some(t) = self.world.get_mut::<Transform>(e) {
                let next = t.pos2() + vel * dt;
                t.set_pos2(next);
            }
        }
        for e in dead {
            self.world.despawn(e);
        }
    }

    fn update_camera(&mut self, dt: f32) {
        let p_pos = self
            .world
            .get::<Transform>(self.player_entity)
            .map(|t| t.pos2())
            .unwrap_or(Vec2::ZERO);
        let b_pos = self
            .world
            .get::<Transform>(self.boss_entity)
            .map(|t| t.pos2())
            .unwrap_or(Vec2::ZERO);
        let focus = p_pos.lerp(b_pos, 0.35) * 0.45;
        let current = self.camera.position.xy();
        let smoothed = current.lerp(focus, (dt * 7.0).min(1.0));
        self.camera.position.x = smoothed.x;
        self.camera.position.y = smoothed.y;
    }

    pub fn render(&mut self, canvas: &mut SoftwareCanvas) {
        self.queue.clear();

        // 1. Optional 3D perspective arena backdrop + rotating crystal monoliths
        if self.camera.mode == ProjectionMode::Hybrid25D {
            self.queue.draw_grid_plane_3d(
                -150.0,
                700.0,
                140.0,
                Color::rgba(46, 62, 98, 120),
                -20.0,
            );
            let t = self.elapsed as f32;
            let corners = [
                Vec3::new(-490.0, -30.0, -310.0),
                Vec3::new(490.0, -30.0, -310.0),
                Vec3::new(-490.0, -30.0, 310.0),
                Vec3::new(490.0, -30.0, 310.0),
            ];
            for (idx, pos) in corners.iter().enumerate() {
                self.queue.draw_wire_box_3d(
                    *pos,
                    Vec3::new(26.0, 72.0, 26.0),
                    t * 0.6 + idx as f32 * 0.8,
                    Color::rgba(56, 225, 245, 110),
                    -18.0,
                );
            }
        }

        // 2. 2D Arena Floor & Neon Perimeter
        self.queue.draw_rect_2d(
            Vec2::ZERO,
            self.arena_half_size * 2.0,
            0.0,
            Color::rgba(20, 23, 34, 225),
            Some(Color::STEEL),
            -5.0,
        );
        let step = 70.0f32;
        let mut gx = -self.arena_half_size.x + step;
        while gx < self.arena_half_size.x {
            self.queue.draw_line_2d(
                Vec2::new(gx, -self.arena_half_size.y),
                Vec2::new(gx, self.arena_half_size.y),
                1.0,
                Color::DARK_GRID,
                -4.0,
            );
            gx += step;
        }
        let mut gy = -self.arena_half_size.y + step;
        while gy < self.arena_half_size.y {
            self.queue.draw_line_2d(
                Vec2::new(-self.arena_half_size.x, gy),
                Vec2::new(self.arena_half_size.x, gy),
                1.0,
                Color::DARK_GRID,
                -4.0,
            );
            gy += step;
        }

        // 3. Boss Visuals
        let boss_pos = self
            .world
            .get::<Transform>(self.boss_entity)
            .map(|t| t.pos2())
            .unwrap_or(Vec2::ZERO);
        let boss_hp = self.world.get::<Health>(self.boss_entity).cloned();
        let boss_comp = self.world.get::<Boss>(self.boss_entity).cloned();
        let script_info = self.world.get::<ScriptComponent>(self.boss_entity).map(|s| {
            (
                s.reload_count,
                s.wait_remaining,
                s.status.clone(),
                s.last_error.clone(),
                s.meow_flash_timer,
            )
        });

        if let (Some(hp), Some(boss)) = (&boss_hp, &boss_comp) {
            self.queue.draw_circle_2d(
                boss_pos + Vec2::new(0.0, 12.0),
                24.0,
                Color::rgba(0, 0, 0, 110),
                None,
                0.5,
            );

            if boss.state == "windup" {
                let pulse = 34.0 + boss.telegraph_pulse.sin().abs() * 14.0;
                self.queue
                    .draw_ring_2d(boss_pos, pulse, 3.0, Color::AMBER, 1.5);
            } else if boss.phase == 2 && !hp.is_dead() {
                self.queue.draw_ring_2d(
                    boss_pos,
                    36.0 + (self.elapsed as f32 * 5.0).sin() * 4.0,
                    2.0,
                    Color::CRIMSON.with_alpha(170),
                    1.5,
                );
            }

            if let Some((_, _, _, _, meow_timer)) = &script_info {
                if *meow_timer > 0.0 {
                    self.queue.draw_ring_2d(
                        boss_pos,
                        46.0,
                        3.5,
                        Color::MAGENTA,
                        1.8,
                    );
                }
            }

            let body_color = if hp.is_dead() {
                Color::SLATE
            } else if hp.flash_timer > 0.0 {
                Color::WHITE
            } else if boss.state == "windup" {
                Color::AMBER
            } else if boss.phase == 2 {
                Color::MAGENTA
            } else {
                Color::CRIMSON
            };
            let spin = if hp.is_dead() {
                0.3
            } else if boss.phase == 2 {
                self.elapsed as f32 * 2.2
            } else {
                self.elapsed as f32 * 0.65
            };
            self.queue.draw_rect_2d(
                boss_pos,
                Vec2::new(46.0, 46.0),
                spin,
                body_color,
                Some(Color::WHITE),
                2.0,
            );
            self.queue.draw_rect_2d(
                boss_pos,
                Vec2::new(22.0, 22.0),
                -spin * 1.4,
                Color::OBSIDIAN,
                Some(Color::AMBER),
                2.1,
            );
        }

        // 4. Player Visuals
        let player_pos = self
            .world
            .get::<Transform>(self.player_entity)
            .map(|t| t.pos2())
            .unwrap_or(Vec2::ZERO);
        let player_hp = self.world.get::<Health>(self.player_entity).cloned();
        let player_comp = self.world.get::<Player>(self.player_entity).cloned();
        if let (Some(hp), Some(pl)) = (&player_hp, &player_comp) {
            self.queue.draw_circle_2d(
                player_pos + Vec2::new(0.0, 10.0),
                16.0,
                Color::rgba(0, 0, 0, 100),
                None,
                0.5,
            );
            if pl.dash_timer > 0.0 {
                self.queue
                    .draw_ring_2d(player_pos, 26.0, 2.5, Color::CYAN, 1.7);
            }
            let p_color = if hp.is_dead() {
                Color::SLATE
            } else if hp.flash_timer > 0.0 {
                Color::WHITE
            } else {
                Color::CYAN
            };
            let angle = pl.facing.y.atan2(pl.facing.x);
            self.queue.draw_rect_2d(
                player_pos,
                Vec2::new(30.0, 30.0),
                angle,
                p_color,
                Some(Color::WHITE),
                2.2,
            );
            self.queue.draw_line_2d(
                player_pos,
                player_pos + pl.facing * 24.0,
                3.0,
                Color::WHITE,
                2.4,
            );
        }

        // 5. Active Hitboxes
        for hb_entity in self.world.entities_with::<Hitbox>() {
            if let (Some(t), Some(hb)) = (
                self.world.get::<Transform>(hb_entity),
                self.world.get::<Hitbox>(hb_entity),
            ) {
                let base = if hb.team == Team::Player {
                    Color::CYAN
                } else {
                    Color::CRIMSON
                };
                let alpha = ((hb.lifetime / hb.max_lifetime.max(0.01)) * 120.0) as u8;
                let outline = if self.show_debug_hitboxes {
                    Some(base)
                } else {
                    None
                };
                self.queue.draw_rect_2d(
                    t.pos2(),
                    hb.size,
                    0.0,
                    base.with_alpha(alpha.max(35)),
                    outline,
                    3.0,
                );
            }
        }

        // 6. Particles
        for p_entity in self.world.entities_with::<Particle>() {
            if let (Some(t), Some(p)) = (
                self.world.get::<Transform>(p_entity),
                self.world.get::<Particle>(p_entity),
            ) {
                let ratio = (p.lifetime / p.max_lifetime.max(0.01)).clamp(0.0, 1.0);
                self.queue.draw_circle_2d(
                    t.pos2(),
                    p.size * ratio.max(0.3),
                    p.color.with_alpha((240.0 * ratio) as u8),
                    None,
                    4.0,
                );
            }
        }

        // 7. HUD Overlay
        self.build_hud(canvas.width as i32, canvas.height as i32, &boss_hp, &boss_comp, &player_hp, &player_comp, &script_info);

        canvas.render_frame(&self.queue, &self.camera, Color::OBSIDIAN);
    }

    #[allow(clippy::too_many_arguments)]
    fn build_hud(
        &mut self,
        w: i32,
        h: i32,
        boss_hp: &Option<Health>,
        boss_comp: &Option<Boss>,
        player_hp: &Option<Health>,
        player_comp: &Option<Player>,
        script_info: &Option<(u32, f64, String, Option<String>, f32)>,
    ) {
        // Top-left Engine Telemetry
        self.queue.hud_rect(
            14,
            14,
            306,
            56,
            Color::rgba(16, 20, 30, 215),
            Some(Color::STEEL),
        );
        self.queue.hud_text(
            24,
            22,
            "FERRIUM ENGINE v0.1 | ECS + FUSTY",
            1,
            Color::CYAN,
        );
        let mode_label = if self.camera.mode == ProjectionMode::Hybrid25D {
            "2.5D+3D"
        } else {
            "2D"
        };
        self.queue.hud_text(
            24,
            36,
            format!(
                "FPS:{:>3.0} ENT:{} MODE:{}",
                self.fps,
                self.world.entity_count(),
                mode_label
            ),
            1,
            Color::WHITE,
        );
        self.queue.hud_text(
            24,
            50,
            format!("FRAME ARENA RESETS: {}", self.frame_arena_resets),
            1,
            Color::EMERALD,
        );

        // Top-center Boss HP Bar
        if let (Some(hp), Some(boss)) = (boss_hp, boss_comp) {
            let bar_w = (w - 680).max(260).min(420);
            let bar_x = (w - bar_w) / 2;
            self.queue.hud_rect(
                bar_x - 8,
                14,
                bar_w + 16,
                44,
                Color::rgba(16, 20, 30, 215),
                Some(if boss.phase == 2 { Color::MAGENTA } else { Color::CRIMSON }),
            );
            self.queue.hud_text(
                bar_x,
                21,
                format!("FUSTY BOSS  PH:{}  [{}]", boss.phase, boss.state.to_uppercase()),
                1,
                Color::WHITE,
            );
            let fill = ((hp.current.max(0) as f32 / hp.max.max(1) as f32) * bar_w as f32) as i32;
            self.queue.hud_rect(bar_x, 35, bar_w, 14, Color::SLATE, Some(Color::BLACK));
            let fill_color = if boss.phase == 2 {
                Color::MAGENTA
            } else {
                Color::CRIMSON
            };
            self.queue.hud_rect(bar_x, 35, fill, 14, fill_color, None);
            self.queue.hud_text(
                bar_x + 6,
                38,
                format!("{}/{}", hp.current, hp.max),
                1,
                Color::WHITE,
            );
        }

        // Top-right FustyLang VM & Hot-Reload Inspector
        let panel_w = 328;
        let panel_x = w - panel_w - 14;
        self.queue.hud_rect(
            panel_x,
            14,
            panel_w,
            154,
            Color::rgba(16, 20, 30, 220),
            Some(Color::STEEL),
        );
        let file_name = self
            .script_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("boss_arena.fy");
        self.queue.hud_text(
            panel_x + 10,
            22,
            format!("FUSTY VM: {} (HOT-RELOAD)", file_name),
            1,
            Color::AMBER,
        );
        if let Some((reloads, wait_rem, status, last_err, meow_timer)) = script_info {
            self.queue.hud_text(
                panel_x + 10,
                38,
                format!("RELOADS: {}  WAIT: {:.2}s", reloads, wait_rem),
                1,
                Color::WHITE,
            );
            let meow_tag = if *meow_timer > 0.0 {
                "MEOW: ACTIVE!"
            } else {
                "MEOW: SAFE"
            };
            let meow_col = if *meow_timer > 0.0 {
                Color::MAGENTA
            } else {
                Color::EMERALD
            };
            self.queue.hud_text(
                panel_x + 10,
                52,
                format!("VM: {} | {}", status, meow_tag),
                1,
                meow_col,
            );
            if let Some(err) = last_err {
                let short_err: String = err.chars().take(48).collect();
                self.queue.hud_text(panel_x + 10, 68, short_err, 1, Color::CRIMSON);
            } else {
                self.queue.hud_text(panel_x + 10, 68, "COMPILE: OK (ZERO GC)", 1, Color::EMERALD);
            }
        }
        for (i, line) in self.logs.iter().rev().take(4).enumerate() {
            let trimmed: String = line.chars().take(48).collect();
            self.queue.hud_text(
                panel_x + 10,
                88 + i as i32 * 15,
                format!("> {}", trimmed),
                1,
                Color::STEEL,
            );
        }

        // Bottom-left Player HP & Dash
        if let (Some(hp), Some(pl)) = (player_hp, player_comp) {
            self.queue.hud_rect(
                14,
                h - 72,
                250,
                58,
                Color::rgba(16, 20, 30, 215),
                Some(Color::CYAN),
            );
            self.queue.hud_text(
                24,
                h - 64,
                format!("PLAYER HP: {}/{}", hp.current, hp.max),
                1,
                Color::WHITE,
            );
            let hp_fill = ((hp.current.max(0) as f32 / hp.max.max(1) as f32) * 228.0) as i32;
            self.queue.hud_rect(24, h - 50, 228, 12, Color::SLATE, None);
            self.queue.hud_rect(24, h - 50, hp_fill, 12, Color::EMERALD, None);

            let dash_ready = 1.0 - (pl.dash_cooldown / 0.65).clamp(0.0, 1.0);
            self.queue.hud_rect(24, h - 32, 228, 8, Color::SLATE, None);
            self.queue.hud_rect(
                24,
                h - 32,
                (dash_ready * 228.0) as i32,
                8,
                Color::CYAN,
                None,
            );
        }

        // Bottom Controls Bar
        self.queue.hud_rect(
            278,
            h - 36,
            w - 292,
            22,
            Color::rgba(16, 20, 30, 200),
            Some(Color::SLATE),
        );
        self.queue.hud_text(
            288,
            h - 29,
            "WASD:MOVE | SPACE/LMB:SLASH | SHIFT/K:DASH | F1:HITBOX | F2:2.5D+3D | F5:RELOAD | R:RESET",
            1,
            Color::WHITE,
        );

        // Victory / Defeat Banner
        let boss_dead = boss_hp.as_ref().map(|h| h.is_dead()).unwrap_or(false);
        let player_dead = player_hp.as_ref().map(|h| h.is_dead()).unwrap_or(false);
        if boss_dead || player_dead {
            let msg = if boss_dead {
                "BOSS DEFEATED! PRESS R TO RESTART"
            } else {
                "YOU FELL IN COMBAT! PRESS R TO RETRY"
            };
            let banner_col = if boss_dead { Color::EMERALD } else { Color::CRIMSON };
            self.queue.hud_rect(
                w / 2 - 230,
                h / 2 - 28,
                460,
                56,
                Color::rgba(10, 12, 18, 235),
                Some(banner_col),
            );
            self.queue.hud_text(w / 2 - 208, h / 2 - 7, msg, 2, banner_col);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_arena_simulation_and_phase2_meow() {
        let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/boss_arena.fy");
        let mut game = FerriumGame::new(script_path);
        let mut canvas = SoftwareCanvas::new(1120, 680);

        // Run 240 ticks (4.0 seconds); at tick 90 drop boss HP to trigger Phase 2 meow blink!
        for tick_idx in 0..240 {
            let mut input = InputState::default();
            if tick_idx == 90 {
                if let Some(hp) = game.world.get_mut::<Health>(game.boss_entity) {
                    hp.current = 450;
                }
            }
            if tick_idx % 20 == 0 {
                input.right = true;
                input.attack_just_pressed = true;
            }
            game.tick(1.0 / 60.0, &input);
        }

        game.render(&mut canvas);

        // Hitstop freezes combat & script ticks briefly on impacts, so resets <= 240.
        assert!(game.frame_arena_resets >= 200 && game.frame_arena_resets <= 240);
        assert!(game.total_hitboxes_spawned > 0);
        assert!(game.saw_meow_teleport, "Boss should trigger Phase 2 meow raw_teleport");

        let script = game.world.get::<ScriptComponent>(game.boss_entity).unwrap();
        assert!(script.last_error.is_none(), "Unexpected script error: {:?}", script.last_error);

        let bg = Color::OBSIDIAN.to_u32_rgb();
        let drawn_pixels = canvas.pixels.iter().filter(|&&p| p != bg).count();
        assert!(drawn_pixels > 50_000, "Expected rich frame output, got {} px", drawn_pixels);
    }
}
