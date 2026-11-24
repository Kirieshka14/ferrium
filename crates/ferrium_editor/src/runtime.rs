use std::collections::HashMap;
use std::f32::consts::TAU;
use std::path::Path;
use std::sync::Arc;

use ferrium_ecs::{Entity, World};
use ferrium_math::{Color, Transform, Vec2, Vec3};
use ferrium_render::{
    shape_2d_polygon, Camera, Mesh3D, RenderBackend, RenderQueue, SoftwareCanvas, Sprite2D,
};
use ferrium_script::{FerriumHost, ScriptCommand, ScriptComponent, ScriptSnapshot, VmResult};

use crate::scene::{EntityKind, MeshKind, SceneEntityDef, SceneFile, TeamDef};

/// Shared cache of procedural & disk-loaded 3D `.obj` meshes and 2D `.png` sprites.
pub struct ModelCache {
    cube: Arc<Mesh3D>,
    sphere: Arc<Mesh3D>,
    cylinder: Arc<Mesh3D>,
    pyramid: Arc<Mesh3D>,
    torus: Arc<Mesh3D>,
    default_sprite: Arc<Sprite2D>,
    obj_meshes: HashMap<String, Option<Arc<Mesh3D>>>,
    sprites: HashMap<String, Option<Arc<Sprite2D>>>,
}

impl Default for ModelCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelCache {
    pub fn new() -> Self {
        Self {
            cube: Arc::new(Mesh3D::cube()),
            sphere: Arc::new(Mesh3D::sphere(10, 16)),
            cylinder: Arc::new(Mesh3D::cylinder(16)),
            pyramid: Arc::new(Mesh3D::pyramid()),
            torus: Arc::new(Mesh3D::torus(16, 10)),
            default_sprite: Arc::new(Sprite2D::procedural_emblem(
                48,
                Color::rgb(56, 130, 246),
                Color::AMBER,
            )),
            obj_meshes: HashMap::new(),
            sprites: HashMap::new(),
        }
    }

    pub fn mesh_for(
        &mut self,
        kind: MeshKind,
        asset_path: Option<&str>,
        project_root: &Path,
    ) -> Option<Arc<Mesh3D>> {
        match kind {
            MeshKind::Cube => Some(Arc::clone(&self.cube)),
            MeshKind::Sphere => Some(Arc::clone(&self.sphere)),
            MeshKind::Cylinder => Some(Arc::clone(&self.cylinder)),
            MeshKind::Pyramid => Some(Arc::clone(&self.pyramid)),
            MeshKind::Torus => Some(Arc::clone(&self.torus)),
            MeshKind::ObjModel => {
                let Some(rel) = asset_path.map(str::trim).filter(|s| !s.is_empty()) else {
                    return Some(Arc::clone(&self.cube));
                };
                if let Some(cached) = self.obj_meshes.get(rel) {
                    return cached.clone().or_else(|| Some(Arc::clone(&self.cube)));
                }
                let p = Path::new(rel);
                let full = if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    project_root.join(p)
                };
                let loaded = Mesh3D::from_obj_file(&full).ok().map(Arc::new);
                self.obj_meshes.insert(rel.to_string(), loaded.clone());
                loaded.or_else(|| Some(Arc::clone(&self.cube)))
            }
            _ => None,
        }
    }

    pub fn sprite_for(&mut self, asset_path: Option<&str>, project_root: &Path) -> Arc<Sprite2D> {
        let Some(rel) = asset_path.map(str::trim).filter(|s| !s.is_empty()) else {
            return Arc::clone(&self.default_sprite);
        };
        if let Some(cached) = self.sprites.get(rel) {
            return cached
                .clone()
                .unwrap_or_else(|| Arc::clone(&self.default_sprite));
        }
        let p = Path::new(rel);
        let full = if p.is_absolute() {
            p.to_path_buf()
        } else {
            project_root.join(p)
        };
        let loaded = Sprite2D::from_png_file(&full).ok().map(Arc::new);
        self.sprites.insert(rel.to_string(), loaded.clone());
        loaded.unwrap_or_else(|| Arc::clone(&self.default_sprite))
    }

    pub fn clear_disk_cache(&mut self) {
        self.obj_meshes.clear();
        self.sprites.clear();
    }

    pub fn stats_for(
        &mut self,
        kind: MeshKind,
        asset_path: Option<&str>,
        project_root: &Path,
    ) -> Option<(usize, usize)> {
        if kind == MeshKind::Plane {
            return Some((4, 2));
        }
        if kind.is_2d() {
            let verts = match kind {
                MeshKind::Quad2D | MeshKind::Sprite2D => 4,
                MeshKind::Triangle2D => 3,
                MeshKind::Star2D => 10,
                MeshKind::Hexagon2D => 6,
                MeshKind::Circle2D => 20,
                _ => 4,
            };
            return Some((verts, verts.saturating_sub(2).max(1)));
        }
        self.mesh_for(kind, asset_path, project_root)
            .map(|m| (m.vertices.len(), m.triangles.len()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleLevel {
    Info,
    Script,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct ConsoleEntry {
    pub timestamp: String,
    pub level: ConsoleLevel,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl RuntimeInput {
    pub fn axis(&self) -> (f64, f64) {
        let mut x = 0.0;
        let mut y = 0.0;
        if self.left {
            x -= 1.0;
        }
        if self.right {
            x += 1.0;
        }
        if self.up {
            y -= 1.0;
        }
        if self.down {
            y += 1.0;
        }
        (x, y)
    }
}

#[derive(Debug, Clone)]
pub struct MetaComp {
    pub name: String,
    pub active: bool,
    pub state_label: String,
}

#[derive(Debug, Clone)]
pub struct MeshComp {
    pub mesh: MeshKind,
    pub asset_path: Option<String>,
    pub wireframe: bool,
    pub color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct VelocityComp(pub Vec2);

#[derive(Debug, Clone)]
pub struct BoxColliderComp {
    pub size: Vec3,
    pub center: Vec3,
}

#[derive(Debug, Clone)]
pub struct HealthComp {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Clone)]
pub struct ParticleComp {
    pub pos: Vec3,
    pub vel: Vec3,
    pub lifetime: f32,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct ScriptTelemetry {
    pub last_error: Option<String>,
}

pub struct EditorRuntime {
    pub world: World,
    pub camera: Camera,
    pub queue: RenderQueue,
    pub elapsed: f64,
    pub frame_arena_resets: u64,
    pub player_entity: Option<Entity>,
    pub light_dir: Vec3,
}

impl EditorRuntime {
    pub fn from_scene(
        scene: &SceneFile,
        project_root: &Path,
        camera_template: &Camera,
        logs: &mut Vec<ConsoleEntry>,
    ) -> Self {
        let mut world = World::new();
        let mut player_entity = None;
        let mut light_dir = Vec3::new(0.65, 1.0, 0.55);

        for def in &scene.entities {
            if def.kind == EntityKind::DirectionalLight && def.active {
                let rx = def.rotation[0].to_radians();
                let ry = def.rotation[1].to_radians();
                light_dir =
                    Vec3::new(ry.sin() * rx.cos(), rx.sin().abs().max(0.35), ry.cos() * rx.cos())
                        .normalized();
            }

            let e = world.spawn();
            world.insert(
                e,
                Transform {
                    position: Vec3::new(def.position[0], def.position[1], def.position[2]),
                    rotation: Vec3::new(
                        def.rotation[0].to_radians(),
                        def.rotation[1].to_radians(),
                        def.rotation[2].to_radians(),
                    ),
                    scale: Vec3::new(def.scale[0], def.scale[1], def.scale[2]),
                },
            );
            world.insert(e, VelocityComp(Vec2::ZERO));
            world.insert(
                e,
                MetaComp {
                    name: def.name.clone(),
                    active: def.active,
                    state_label: "idle".into(),
                },
            );

            if let (Some(mf), Some(mr)) = (&def.mesh_filter, &def.mesh_renderer) {
                world.insert(
                    e,
                    MeshComp {
                        mesh: mf.mesh,
                        asset_path: mf.asset_path.clone(),
                        wireframe: mf.wireframe,
                        color: mr.color.clone().to_ferrium(),
                    },
                );
            }
            if let Some(bc) = &def.box_collider {
                if bc.team == TeamDef::Player {
                    player_entity = Some(e);
                }
                world.insert(
                    e,
                    BoxColliderComp {
                        size: Vec3::new(bc.size[0], bc.size[1], bc.size[2]),
                        center: Vec3::new(bc.center[0], bc.center[1], bc.center[2]),
                    },
                );
            }
            if let Some(hp) = &def.health {
                world.insert(
                    e,
                    HealthComp {
                        current: hp.current,
                        max: hp.max,
                    },
                );
            }
            if def.name == "Player" && player_entity.is_none() {
                player_entity = Some(e);
            }

            if def.active {
                if let Some(sc) = &def.script {
                    if sc.enabled && !sc.path.trim().is_empty() {
                        let p = Path::new(&sc.path);
                        let resolved = if p.is_absolute() {
                            p.to_path_buf()
                        } else {
                            project_root.join(p)
                        };
                        match ScriptComponent::from_file(&resolved) {
                            Ok(comp) => {
                                logs.push(ConsoleEntry {
                                    timestamp: format_clock(0.0),
                                    level: ConsoleLevel::Info,
                                    text: format!("Loaded '{}' on {}", sc.path, def.name),
                                });
                                world.insert(e, comp);
                            }
                            Err(err) => {
                                logs.push(ConsoleEntry {
                                    timestamp: format_clock(0.0),
                                    level: ConsoleLevel::Error,
                                    text: format!("Script error on {}: {}", def.name, err),
                                });
                            }
                        }
                    }
                }
            }
        }

        let mut camera = camera_template.clone();
        camera.use_orbit_3d = true;

        Self {
            world,
            camera,
            queue: RenderQueue::new(),
            elapsed: 0.0,
            frame_arena_resets: 0,
            player_entity,
            light_dir,
        }
    }

    pub fn force_reload_all(&mut self, logs: &mut Vec<ConsoleEntry>) {
        for e in self.world.entities_with::<ScriptComponent>() {
            let name = self
                .world
                .get::<MetaComp>(e)
                .map(|m| m.name.clone())
                .unwrap_or_else(|| "Entity".into());
            if let Some(sc) = self.world.get_mut::<ScriptComponent>(e) {
                match sc.try_hot_reload(true) {
                    Some(Ok(n)) => logs.push(ConsoleEntry {
                        timestamp: format_clock(self.elapsed),
                        level: ConsoleLevel::Info,
                        text: format!("[{}] Hot-reloaded #{}", name, n),
                    }),
                    Some(Err(err)) => logs.push(ConsoleEntry {
                        timestamp: format_clock(self.elapsed),
                        level: ConsoleLevel::Error,
                        text: format!("[{}] Reload error: {}", name, err),
                    }),
                    None => {}
                }
            }
        }
    }

    pub fn collect_script_telemetry(&self) -> Vec<ScriptTelemetry> {
        self.world
            .entities_slice::<ScriptComponent>()
            .iter()
            .filter_map(|&e| {
                self.world.get::<ScriptComponent>(e).map(|s| ScriptTelemetry {
                    last_error: s.last_error.clone(),
                })
            })
            .collect()
    }

    pub fn step(&mut self, dt: f32, input: &RuntimeInput, logs: &mut Vec<ConsoleEntry>) {
        let dt = dt.clamp(0.001, 0.05);
        self.elapsed += dt as f64;
        self.camera.update(dt, self.elapsed as f32);

        self.update_scripts(dt, input, logs);
        self.integrate_physics(dt);
        self.update_particles(dt);

        if logs.len() > 250 {
            let drain = logs.len() - 250;
            logs.drain(0..drain);
        }
    }

    fn update_scripts(&mut self, dt: f32, input: &RuntimeInput, logs: &mut Vec<ConsoleEntry>) {
        let (ix, iy) = input.axis();
        let (player_pos, player_hp) = match self.player_entity {
            Some(pe) => (
                self.world
                    .get::<Transform>(pe)
                    .map(|t| Vec2::new(t.position.x, t.position.z))
                    .unwrap_or(Vec2::ZERO),
                self.world
                    .get::<HealthComp>(pe)
                    .map(|h| h.current)
                    .unwrap_or(100),
            ),
            None => (Vec2::ZERO, 100),
        };

        let mut deferred = Vec::new();
        for entity in self.world.entities_with::<ScriptComponent>() {
            let (name, active) = self
                .world
                .get::<MetaComp>(entity)
                .map(|m| (m.name.clone(), m.active))
                .unwrap_or_else(|| ("Entity".into(), true));
            if !active {
                continue;
            }
            let (self_pos, self_pos3, self_rot, self_scale) = self
                .world
                .get::<Transform>(entity)
                .map(|t| {
                    (
                        Vec2::new(t.position.x, t.position.z),
                        t.position,
                        t.rotation,
                        t.scale,
                    )
                })
                .unwrap_or((Vec2::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ONE));
            let self_vel = self
                .world
                .get::<VelocityComp>(entity)
                .map(|v| v.0)
                .unwrap_or(Vec2::ZERO);
            let (self_hp, self_max_hp) = self
                .world
                .get::<HealthComp>(entity)
                .map(|h| (h.current, h.max))
                .unwrap_or((100, 100));

            let mut used_meow = false;
            if let Some(script) = self.world.get_mut::<ScriptComponent>(entity) {
                script.meow_flash_timer = (script.meow_flash_timer - dt).max(0.0);
                script.poll_timer += dt as f64;
                if script.poll_timer >= 0.25 {
                    script.poll_timer = 0.0;
                    if let Some(res) = script.try_hot_reload(false) {
                        match res {
                            Ok(n) => logs.push(ConsoleEntry {
                                timestamp: format_clock(self.elapsed),
                                level: ConsoleLevel::Info,
                                text: format!("[{}] Hot-reloaded #{}", name, n),
                            }),
                            Err(err) => logs.push(ConsoleEntry {
                                timestamp: format_clock(self.elapsed),
                                level: ConsoleLevel::Error,
                                text: format!("[{}] Reload err: {}", name, err),
                            }),
                        }
                    }
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
                        input_x: ix,
                        input_y: iy,
                        dt: dt as f64,
                        elapsed_time: self.elapsed,
                    };
                    let mut host = FerriumHost::new(snapshot);
                    match script.vm.run_with_host(&script.chunk, &mut host) {
                        Ok(VmResult::Waiting { duration_sec }) => {
                            script.wait_remaining = duration_sec;
                            script.status = format!("wait {:.2}s", duration_sec);
                        }
                        Ok(VmResult::Yielded) => script.status = "yield".into(),
                        Ok(VmResult::Done(_)) => script.status = "done".into(),
                        Err(e) => {
                            let msg = format!("[{}] VM line {}: {}", name, e.line, e.message);
                            script.last_error = Some(msg.clone());
                            script.status = "error".into();
                            logs.push(ConsoleEntry {
                                timestamp: format_clock(self.elapsed),
                                level: ConsoleLevel::Error,
                                text: msg,
                            });
                        }
                    }
                    used_meow = host.used_meow;
                    deferred.extend(host.commands);
                }

                script.vm.reset_frame();
                self.frame_arena_resets += 1;
                if used_meow {
                    script.meow_flash_timer = 0.75;
                }
            }
        }

        for cmd in deferred {
            match cmd {
                ScriptCommand::SetPos(e, pos) | ScriptCommand::RawTeleport(e, pos) => {
                    if let Some(t) = self.world.get_mut::<Transform>(e) {
                        t.position.x = pos.x.clamp(-8.0, 8.0);
                        t.position.z = pos.y.clamp(-8.0, 8.0);
                    }
                }
                ScriptCommand::SetPos3(e, pos) | ScriptCommand::RawTeleport3(e, pos) => {
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
                        t.scale = Vec3::new(scale.x.max(0.05), scale.y.max(0.05), scale.z.max(0.05));
                    }
                }
                ScriptCommand::SetColor(e, col) => {
                    if let Some(mc) = self.world.get_mut::<MeshComp>(e) {
                        mc.color = col;
                    }
                }
                ScriptCommand::SetVel(e, vel) => {
                    if let Some(v) = self.world.get_mut::<VelocityComp>(e) {
                        v.0 = vel;
                    }
                }
                ScriptCommand::SetState { entity, state } => {
                    if let Some(m) = self.world.get_mut::<MetaComp>(entity) {
                        m.state_label = state;
                    }
                }
                ScriptCommand::CameraShake { intensity, duration } => {
                    self.camera.trigger_shake(intensity, duration);
                }
                ScriptCommand::SpawnParticles { pos, count, color } => {
                    for i in 0..count {
                        let a = (i as f32 / count.max(1) as f32) * TAU;
                        let pe = self.world.spawn();
                        self.world.insert(
                            pe,
                            ParticleComp {
                                pos: Vec3::new(pos.x, 0.5, pos.y),
                                vel: Vec3::new(a.cos() * 1.8, 1.4, a.sin() * 1.8),
                                lifetime: 0.45,
                                color,
                            },
                        );
                    }
                }
                ScriptCommand::Log(msg) => {
                    logs.push(ConsoleEntry {
                        timestamp: format_clock(self.elapsed),
                        level: ConsoleLevel::Script,
                        text: msg,
                    });
                }
                ScriptCommand::SpawnHitbox { .. } => {}
            }
        }
    }

    fn integrate_physics(&mut self, dt: f32) {
        for e in self.world.entities_with::<VelocityComp>() {
            let vel = self
                .world
                .get::<VelocityComp>(e)
                .map(|v| v.0)
                .unwrap_or(Vec2::ZERO);
            if vel.length_sq() < 1e-8 {
                continue;
            }
            if let Some(t) = self.world.get_mut::<Transform>(e) {
                t.position.x = (t.position.x + vel.x * dt).clamp(-8.0, 8.0);
                t.position.z = (t.position.z + vel.y * dt).clamp(-8.0, 8.0);
            }
        }
    }

    fn update_particles(&mut self, dt: f32) {
        let mut dead = Vec::new();
        if let Some(storage) = self.world.storage_mut::<ParticleComp>() {
            for (e, p) in storage.iter_mut() {
                p.lifetime -= dt;
                p.pos = p.pos + p.vel * dt;
                p.vel.y -= 3.5 * dt;
                if p.lifetime <= 0.0 {
                    dead.push(e);
                }
            }
        }
        for e in dead {
            self.world.despawn(e);
        }
    }

    pub fn render_play_viewport(
        &mut self,
        show_grid: bool,
        show_gizmos: bool,
        view_2d_mode: bool,
        cache: &mut ModelCache,
        project_root: &Path,
        canvas: &mut SoftwareCanvas,
    ) {
        self.queue.clear();
        if view_2d_mode {
            push_2d_backdrop(&mut self.queue, show_grid);
        } else {
            push_studio_backdrop(&mut self.queue, show_grid);
        }

        let eye = self.camera.orbit_eye();
        for &e in self.world.entities_slice::<MeshComp>() {
            if self.world.get::<MetaComp>(e).map(|m| !m.active).unwrap_or(false) {
                continue;
            }
            let t = self.world.get::<Transform>(e).unwrap();
            let mc = self.world.get::<MeshComp>(e).unwrap();
            push_model_command(
                &mut self.queue,
                cache,
                project_root,
                mc.mesh,
                mc.asset_path.as_deref(),
                mc.wireframe,
                t.position,
                t.rotation,
                t.scale,
                mc.color,
                Some(Color::rgb(175, 185, 200)),
                self.light_dir,
                eye,
                view_2d_mode,
            );

            if show_gizmos && !view_2d_mode {
                if let Some(bc) = self.world.get::<BoxColliderComp>(e) {
                    let center = Vec3::new(
                        t.position.x + bc.center.x,
                        t.position.y + bc.size.y * 0.5 + bc.center.y,
                        t.position.z + bc.center.z,
                    );
                    self.queue.draw_wire_box_3d(
                        center,
                        bc.size * 0.505,
                        t.rotation.y,
                        Color::rgba(34, 197, 94, 180),
                        4.0,
                    );
                }
            }
        }

        for &pe in self.world.entities_slice::<ParticleComp>() {
            if let Some(p) = self.world.get::<ParticleComp>(pe) {
                if view_2d_mode {
                    self.queue.draw_circle_2d(
                        Vec2::new(p.pos.x * 80.0, -p.pos.y * 80.0),
                        5.0,
                        p.color,
                        None,
                        5.0,
                    );
                } else {
                    self.queue.draw_solid_box_3d(
                        p.pos,
                        Vec3::new(0.07, 0.07, 0.07),
                        Vec3::ZERO,
                        p.color,
                        None,
                        self.light_dir,
                        2.0,
                    );
                }
            }
        }

        canvas.render_frame(&self.queue, &self.camera, Color::rgb(17, 20, 27));
    }
}

fn push_studio_backdrop(queue: &mut RenderQueue, show_grid: bool) {
    queue.set_sky_gradient(
        Color::rgb(14, 17, 24),
        Color::rgb(35, 42, 56),
        Color::rgb(22, 26, 36),
    );
    if show_grid {
        queue.draw_fade_grid_3d(
            0.0,
            10.0,
            1.0,
            Color::rgba(92, 106, 130, 110),
            Color::rgba(135, 72, 78, 150),
            Color::rgba(72, 110, 165, 150),
            -10.0,
        );
    }
}

fn push_2d_backdrop(queue: &mut RenderQueue, show_grid: bool) {
    if !show_grid {
        return;
    }
    let step = 80.0f32;
    let half = 800.0f32;
    let mut x = -half;
    while x <= half {
        let col = if x.abs() < 1.0 {
            Color::rgba(56, 189, 128, 160) // Y axis green
        } else {
            Color::rgba(46, 54, 72, 120)
        };
        queue.draw_line_2d(Vec2::new(x, -half), Vec2::new(x, half), 1.0, col, -20.0);
        x += step;
    }
    let mut y = -half;
    while y <= half {
        let col = if y.abs() < 1.0 {
            Color::rgba(220, 74, 74, 160) // X axis red
        } else {
            Color::rgba(46, 54, 72, 120)
        };
        queue.draw_line_2d(Vec2::new(-half, y), Vec2::new(half, y), 1.0, col, -20.0);
        y += step;
    }
}

const UNITS_TO_PX_2D: f32 = 80.0;

#[allow(clippy::too_many_arguments)]
fn push_model_command(
    queue: &mut RenderQueue,
    cache: &mut ModelCache,
    project_root: &Path,
    mesh: MeshKind,
    asset_path: Option<&str>,
    wireframe: bool,
    pos: Vec3,
    rot: Vec3,
    scale: Vec3,
    color: Color,
    edge_color: Option<Color>,
    light_dir: Vec3,
    eye: Vec3,
    view_2d_mode: bool,
) {
    if view_2d_mode {
        if mesh == MeshKind::Plane {
            return;
        }
        let center2 = Vec2::new(pos.x * UNITS_TO_PX_2D, -pos.y * UNITS_TO_PX_2D);
        let size2 = Vec2::new(
            scale.x.abs().max(0.1) * UNITS_TO_PX_2D,
            scale.y.abs().max(0.1) * UNITS_TO_PX_2D,
        );
        let z_ord = -pos.z;
        match mesh {
            MeshKind::Circle2D | MeshKind::Sphere => {
                queue.draw_circle_2d(center2, size2.x.min(size2.y) * 0.5, color, edge_color, z_ord);
            }
            MeshKind::Triangle2D | MeshKind::Pyramid => {
                queue.draw_polygon_2d(
                    center2,
                    size2,
                    -rot.z,
                    shape_2d_polygon(1),
                    color,
                    edge_color,
                    z_ord,
                );
            }
            MeshKind::Star2D => {
                queue.draw_polygon_2d(
                    center2,
                    size2,
                    -rot.z,
                    shape_2d_polygon(2),
                    color,
                    edge_color,
                    z_ord,
                );
            }
            MeshKind::Hexagon2D => {
                queue.draw_polygon_2d(
                    center2,
                    size2,
                    -rot.z,
                    shape_2d_polygon(3),
                    color,
                    edge_color,
                    z_ord,
                );
            }
            MeshKind::Sprite2D => {
                let sp = cache.sprite_for(asset_path, project_root);
                queue.draw_sprite_2d(sp, center2, size2, -rot.z, color, edge_color, z_ord);
            }
            _ => {
                queue.draw_rect_2d(center2, size2, -rot.z, color, edge_color, z_ord);
            }
        }
        return;
    }

    // 3D Studio Viewport mode
    match mesh {
        MeshKind::Plane => {
            queue.draw_solid_plane_3d(
                pos,
                Vec2::new(scale.x * 0.5, scale.z * 0.5),
                color,
                if wireframe { edge_color } else { None },
                -8.0,
            );
        }
        MeshKind::Cube => {
            let half = scale * 0.5;
            let center = Vec3::new(pos.x, pos.y + half.y, pos.z);
            let dist = (center - eye).length();
            if wireframe {
                if let Some(m) = cache.mesh_for(MeshKind::Cube, None, project_root) {
                    queue.draw_mesh_3d(m, center, scale, rot, color, edge_color, true, light_dir, -dist);
                }
            } else {
                queue.draw_solid_box_3d(center, half, rot, color, edge_color, light_dir, -dist);
            }
        }
        MeshKind::Sphere
        | MeshKind::Cylinder
        | MeshKind::Pyramid
        | MeshKind::Torus
        | MeshKind::ObjModel => {
            let center = Vec3::new(pos.x, pos.y + scale.y * 0.5, pos.z);
            let dist = (center - eye).length();
            if let Some(m) = cache.mesh_for(mesh, asset_path, project_root) {
                queue.draw_mesh_3d(
                    m,
                    center,
                    scale,
                    rot,
                    color,
                    edge_color,
                    wireframe,
                    light_dir,
                    -dist,
                );
            }
        }
        MeshKind::Quad2D
        | MeshKind::Circle2D
        | MeshKind::Triangle2D
        | MeshKind::Star2D
        | MeshKind::Hexagon2D
        | MeshKind::Sprite2D => {
            let center = Vec3::new(pos.x, pos.y + scale.y * 0.5, pos.z);
            let dist = (center - eye).length();
            let shape_kind = match mesh {
                MeshKind::Quad2D => 0,
                MeshKind::Triangle2D => 1,
                MeshKind::Star2D => 2,
                MeshKind::Hexagon2D => 3,
                MeshKind::Circle2D => 4,
                MeshKind::Sprite2D => 5,
                _ => 0,
            };
            let sprite = if mesh == MeshKind::Sprite2D {
                Some(cache.sprite_for(asset_path, project_root))
            } else {
                None
            };
            queue.draw_model_2d_in_3d(
                shape_kind,
                sprite,
                center,
                Vec2::new(scale.x, scale.y),
                rot,
                color,
                edge_color,
                -dist,
            );
        }
    }
}

pub fn entity_world_center_and_half(ent: &SceneEntityDef) -> (Vec3, Vec3) {
    let pos = Vec3::new(ent.position[0], ent.position[1], ent.position[2]);
    let scale = Vec3::new(
        ent.scale[0].abs().max(0.1),
        ent.scale[1].abs().max(0.1),
        ent.scale[2].abs().max(0.1),
    );
    let mesh = ent.mesh_filter.as_ref().map(|m| m.mesh);
    match mesh {
        Some(MeshKind::Plane) => (pos, Vec3::new(scale.x * 0.5, 0.05, scale.z * 0.5)),
        Some(m) if m.is_2d() => (
            Vec3::new(pos.x, pos.y + scale.y * 0.5, pos.z),
            Vec3::new(scale.x * 0.5, scale.y * 0.5, 0.12),
        ),
        Some(_) => (
            Vec3::new(pos.x, pos.y + scale.y * 0.5, pos.z),
            scale * 0.5,
        ),
        None => (pos, Vec3::new(0.35, 0.35, 0.35)),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_edit_viewport(
    scene: &SceneFile,
    selected_idx: Option<usize>,
    camera: &Camera,
    show_grid: bool,
    show_gizmos: bool,
    active_tool: u8,
    hovered_axis: Option<u8>,
    view_2d_mode: bool,
    queue: &mut RenderQueue,
    cache: &mut ModelCache,
    project_root: &Path,
    canvas: &mut SoftwareCanvas,
) {
    queue.clear();
    if view_2d_mode {
        push_2d_backdrop(queue, show_grid);
    } else {
        push_studio_backdrop(queue, show_grid);
    }

    let light_dir = scene
        .entities
        .iter()
        .find(|e| e.kind == EntityKind::DirectionalLight && e.active)
        .map(|l| {
            let rx = l.rotation[0].to_radians();
            let ry = l.rotation[1].to_radians();
            Vec3::new(ry.sin() * rx.cos(), rx.sin().abs().max(0.35), ry.cos() * rx.cos())
                .normalized()
        })
        .unwrap_or_else(|| Vec3::new(0.65, 1.0, 0.55));

    let eye = camera.orbit_eye();

    for (idx, ent) in scene.entities.iter().enumerate() {
        if !ent.active {
            continue;
        }
        let pos = Vec3::new(ent.position[0], ent.position[1], ent.position[2]);
        let rot = Vec3::new(
            ent.rotation[0].to_radians(),
            ent.rotation[1].to_radians(),
            ent.rotation[2].to_radians(),
        );
        let scale = Vec3::new(ent.scale[0], ent.scale[1], ent.scale[2]);
        let is_selected = Some(idx) == selected_idx;

        if let (Some(mf), Some(mr)) = (&ent.mesh_filter, &ent.mesh_renderer) {
            let col = mr.color.clone().to_ferrium();
            let edge = if is_selected {
                Some(Color::AMBER)
            } else if mf.mesh == MeshKind::Cube || mf.mesh.is_2d() || mf.wireframe {
                Some(Color::rgb(130, 142, 162))
            } else {
                None
            };
            push_model_command(
                queue,
                cache,
                project_root,
                mf.mesh,
                mf.asset_path.as_deref(),
                mf.wireframe,
                pos,
                rot,
                scale,
                col,
                edge,
                light_dir,
                eye,
                view_2d_mode,
            );
        }

        if is_selected && show_gizmos {
            if view_2d_mode {
                let c2 = Vec2::new(pos.x * UNITS_TO_PX_2D, -pos.y * UNITS_TO_PX_2D);
                let ax_len = 60.0 / camera.zoom.max(0.2);
                let col_x = if hovered_axis == Some(0) {
                    Color::AMBER
                } else {
                    Color::rgb(239, 68, 68)
                };
                let col_y = if hovered_axis == Some(1) {
                    Color::AMBER
                } else {
                    Color::rgb(34, 197, 94)
                };
                queue.draw_line_2d(c2, c2 + Vec2::new(ax_len, 0.0), 2.5, col_x, 20.0);
                queue.draw_line_2d(c2, c2 + Vec2::new(0.0, -ax_len), 2.5, col_y, 20.0);
                queue.draw_circle_2d(c2, 5.0 / camera.zoom.max(0.2), Color::WHITE, None, 21.0);
            } else {
                let (center, half) = entity_world_center_and_half(ent);
                if let Some(bc) = &ent.box_collider {
                    let col_center = Vec3::new(
                        pos.x + bc.center[0],
                        pos.y + half.y + bc.center[1],
                        pos.z + bc.center[2],
                    );
                    queue.draw_wire_box_3d(
                        col_center,
                        Vec3::new(bc.size[0] * 0.506, bc.size[1] * 0.506, bc.size[2] * 0.506),
                        rot.y,
                        Color::rgba(34, 197, 94, 160),
                        4.0,
                    );
                }
                let gizmo_scale = (half.x.max(half.y).max(half.z) * 1.4).clamp(0.75, 2.2);
                queue.draw_gizmo_3d_ex(center, gizmo_scale, active_tool, hovered_axis, 10.0);
            }
        }
    }

    canvas.render_frame(queue, camera, Color::rgb(17, 20, 27));
}

/// Ray-OBB 3D entity picking: picks the nearest intersecting 3D or 2D model along the cursor ray.
pub fn pick_entity_3d(
    scene: &SceneFile,
    screen_pos: Vec2,
    width: u32,
    height: u32,
    camera: &Camera,
) -> Option<usize> {
    let ray = camera.screen_ray_3d(screen_pos, width, height);
    let mut best: Option<(usize, f32)> = None;

    for (idx, ent) in scene.entities.iter().enumerate() {
        if !ent.active || ent.kind != EntityKind::Mesh {
            continue;
        }
        if ent
            .mesh_filter
            .as_ref()
            .map(|m| m.mesh == MeshKind::Plane)
            .unwrap_or(false)
        {
            continue;
        }
        let (center, half) = entity_world_center_and_half(ent);
        let rot = Vec3::new(
            ent.rotation[0].to_radians(),
            ent.rotation[1].to_radians(),
            ent.rotation[2].to_radians(),
        );
        if let Some(t) = ray.intersect_obb(center, half, rot) {
            if best.map(|(_, bt)| t < bt).unwrap_or(true) {
                best = Some((idx, t));
            }
        }
    }

    best.map(|(i, _)| i)
}

/// 2D orthographic entity picking in local rotated space.
pub fn pick_entity_2d(
    scene: &SceneFile,
    screen_pos: Vec2,
    width: u32,
    height: u32,
    camera: &Camera,
) -> Option<usize> {
    let world = camera.screen_to_world_2d(screen_pos, width, height);
    let mut best: Option<(usize, f32)> = None;

    for (idx, ent) in scene.entities.iter().enumerate() {
        if !ent.active || ent.kind != EntityKind::Mesh {
            continue;
        }
        if ent
            .mesh_filter
            .as_ref()
            .map(|m| m.mesh == MeshKind::Plane)
            .unwrap_or(false)
        {
            continue;
        }
        let cx = ent.position[0] * UNITS_TO_PX_2D;
        let cy = -ent.position[1] * UNITS_TO_PX_2D;
        let hw = ent.scale[0].abs().max(0.2) * UNITS_TO_PX_2D * 0.5;
        let hh = ent.scale[1].abs().max(0.2) * UNITS_TO_PX_2D * 0.5;
        let angle = -ent.rotation[2].to_radians();
        let (s, c) = angle.sin_cos();
        let dx = world.x - cx;
        let dy = world.y - cy;
        let lx = (dx * c + dy * s).abs();
        let ly = (-dx * s + dy * c).abs();
        if lx <= hw + 4.0 && ly <= hh + 4.0 {
            let z_ord = -ent.position[2];
            if best.map(|(_, bz)| z_ord >= bz).unwrap_or(true) {
                best = Some((idx, z_ord));
            }
        }
    }

    best.map(|(i, _)| i)
}

/// Hit-tests the 3D transform gizmo (X=0, Y=1, Z=2) in screen space.
pub fn pick_gizmo_axis_3d(
    ent: &SceneEntityDef,
    screen_pos: Vec2,
    width: u32,
    height: u32,
    camera: &Camera,
    active_tool: u8,
) -> Option<u8> {
    let (origin, half) = entity_world_center_and_half(ent);
    let scale = (half.x.max(half.y).max(half.z) * 1.4).clamp(0.75, 2.2);
    let (ox, oy, _) = camera.world_to_screen_3d(origin, width, height)?;

    if active_tool == 2 {
        // Rotate rings
        let ring_axes = [
            (0u8, Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
            (1u8, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
            (2u8, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        ];
        let segs = 24;
        let rad = scale * 0.85;
        let mut best: Option<(u8, f32)> = None;
        for (axis_id, u, v) in ring_axes {
            for i in 0..segs {
                let a0 = (i as f32 / segs as f32) * TAU;
                let a1 = ((i + 1) as f32 / segs as f32) * TAU;
                let p0 = origin + (u * a0.cos() + v * a0.sin()) * rad;
                let p1 = origin + (u * a1.cos() + v * a1.sin()) * rad;
                if let (Some((x0, y0, _)), Some((x1, y1, _))) = (
                    camera.world_to_screen_3d(p0, width, height),
                    camera.world_to_screen_3d(p1, width, height),
                ) {
                    let d = point_to_segment_dist(screen_pos, Vec2::new(x0, y0), Vec2::new(x1, y1));
                    if d < 12.0 && best.map(|(_, bd)| d < bd).unwrap_or(true) {
                        best = Some((axis_id, d));
                    }
                }
            }
        }
        return best.map(|(id, _)| id);
    }

    let axes = [
        (0u8, Vec3::new(scale, 0.0, 0.0)),
        (1u8, Vec3::new(0.0, scale, 0.0)),
        (2u8, Vec3::new(0.0, 0.0, scale)),
    ];
    let mut best: Option<(u8, f32)> = None;
    let o2 = Vec2::new(ox, oy);
    for (axis_id, dir) in axes {
        if let Some((ex, ey, _)) = camera.world_to_screen_3d(origin + dir, width, height) {
            let d = point_to_segment_dist(screen_pos, o2, Vec2::new(ex, ey));
            if d < 14.0 && best.map(|(_, bd)| d < bd).unwrap_or(true) {
                best = Some((axis_id, d));
            }
        }
    }
    best.map(|(id, _)| id)
}

/// Hit-tests 2D X/Y gizmo handles in screen space.
pub fn pick_gizmo_axis_2d(
    ent: &SceneEntityDef,
    screen_pos: Vec2,
    width: u32,
    height: u32,
    camera: &Camera,
) -> Option<u8> {
    let c2 = Vec2::new(ent.position[0] * UNITS_TO_PX_2D, -ent.position[1] * UNITS_TO_PX_2D);
    let ax_len = 60.0 / camera.zoom.max(0.2);
    let (ox, oy) = camera.world_to_screen_2d(c2, width, height);
    let (xx, xy) = camera.world_to_screen_2d(c2 + Vec2::new(ax_len, 0.0), width, height);
    let (yx, yy) = camera.world_to_screen_2d(c2 + Vec2::new(0.0, -ax_len), width, height);
    let dx = point_to_segment_dist(screen_pos, Vec2::new(ox, oy), Vec2::new(xx, xy));
    let dy = point_to_segment_dist(screen_pos, Vec2::new(ox, oy), Vec2::new(yx, yy));
    if dx < 12.0 && dx <= dy {
        Some(0)
    } else if dy < 12.0 {
        Some(1)
    } else {
        None
    }
}

fn point_to_segment_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_sq();
    if len_sq < 1e-4 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    let proj = a + ab * t;
    p.distance(proj)
}

pub fn format_clock(elapsed_sec: f64) -> String {
    let total = elapsed_sec.max(0.0) as u64;
    let mins = (total / 60) % 60;
    let secs = total % 60;
    let hours = 14 + (total / 3600) % 10;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_editor_runtime_sample_scene_and_player_script() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let scene = SceneFile::sample_scene();
        let mut cam = Camera::new();
        cam.use_orbit_3d = true;
        let mut logs = Vec::new();
        let mut rt = EditorRuntime::from_scene(&scene, &root, &cam, &mut logs);
        let mut cache = ModelCache::new();

        let start_x = rt
            .world
            .get::<Transform>(rt.player_entity.unwrap())
            .unwrap()
            .position
            .x;

        for _ in 0..30 {
            let input = RuntimeInput {
                right: true,
                ..Default::default()
            };
            rt.step(1.0 / 60.0, &input, &mut logs);
        }

        let end_x = rt
            .world
            .get::<Transform>(rt.player_entity.unwrap())
            .unwrap()
            .position
            .x;
        assert!(end_x > start_x + 0.5, "Player.fy should move cube right on D/Right");
        assert_eq!(rt.frame_arena_resets, 30);

        let telemetry = rt.collect_script_telemetry();
        assert_eq!(telemetry.len(), 1);
        assert!(telemetry[0].last_error.is_none());

        let mut canvas = SoftwareCanvas::new(640, 360);
        rt.render_play_viewport(true, true, false, &mut cache, &root, &mut canvas);
        let bg = Color::rgb(17, 20, 27).to_u32_rgb();
        assert!(canvas.pixels.iter().filter(|&&p| p != bg).count() > 10_000);
    }

    #[test]
    fn test_pick_3d_and_2d_models() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let scene = SceneFile::boss_demo();
        let mut cam = Camera::new();
        cam.use_orbit_3d = true;

        // Player cube sits at (0, 0.5, 0) -> project to screen and ray-pick it
        let player_idx = scene.entities.iter().position(|e| e.name == "Player").unwrap();
        let (sx, sy, _) = cam
            .world_to_screen_3d(Vec3::new(0.0, 0.5, 0.0), 720, 440)
            .unwrap();
        let picked = pick_entity_3d(&scene, Vec2::new(sx, sy), 720, 440, &cam);
        assert_eq!(picked, Some(player_idx));

        // Verify OBJ model + 2D sprite render cleanly in Edit viewport
        let mut queue = RenderQueue::new();
        let mut cache = ModelCache::new();
        let mut canvas = SoftwareCanvas::new(720, 440);
        render_edit_viewport(
            &scene,
            Some(player_idx),
            &cam,
            true,
            true,
            1,
            Some(0),
            false,
            &mut queue,
            &mut cache,
            &root,
            &mut canvas,
        );
        let bg = Color::rgb(17, 20, 27).to_u32_rgb();
        assert!(canvas.pixels.iter().filter(|&&p| p != bg).count() > 10_000);
    }
}
