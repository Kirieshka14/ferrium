use ferrium_ecs::Entity;
use ferrium_math::{Color, Vec2, Vec3};
use fusty_vm::value::{Value, Vec2 as FustyVec2, Vec3 as FustyVec3};
use fusty_vm::vm::Host;

/// Read-only world snapshot passed into a FustyLang VM tick.
#[derive(Debug, Clone)]
pub struct ScriptSnapshot {
    pub self_entity: Entity,
    pub self_pos: Vec2,
    pub self_pos3: Vec3,
    pub self_rot: Vec3,
    pub self_scale: Vec3,
    pub self_vel: Vec2,
    pub self_hp: i32,
    pub self_max_hp: i32,
    pub player_pos: Vec2,
    pub player_hp: i32,
    pub input_x: f64,
    pub input_y: f64,
    pub dt: f64,
    pub elapsed_time: f64,
    pub frame_count: u64,
    pub jit_enabled: bool,
    pub jit_compiled_count: usize,
}

impl Default for ScriptSnapshot {
    fn default() -> Self {
        Self {
            self_entity: Entity {
                id: 0,
                generation: 0,
            },
            self_pos: Vec2::ZERO,
            self_pos3: Vec3::ZERO,
            self_rot: Vec3::ZERO,
            self_scale: Vec3::ONE,
            self_vel: Vec2::ZERO,
            self_hp: 100,
            self_max_hp: 100,
            player_pos: Vec2::ZERO,
            player_hp: 100,
            input_x: 0.0,
            input_y: 0.0,
            dt: 1.0 / 60.0,
            elapsed_time: 0.0,
            frame_count: 0,
            jit_enabled: true,
            jit_compiled_count: 0,
        }
    }
}

/// Deferred ECS mutation or engine side-effect emitted by a FustyLang script.
#[derive(Debug, Clone)]
pub enum ScriptCommand {
    SetPos(Entity, Vec2),
    SetPos3(Entity, Vec3),
    SetRot(Entity, Vec3),
    SetScale(Entity, Vec3),
    SetColor(Entity, Color),
    SetVel(Entity, Vec2),
    RawTeleport(Entity, Vec2),
    RawTeleport3(Entity, Vec3),
    SpawnHitbox {
        owner: Entity,
        offset: Vec2,
        size: Vec2,
        damage: i32,
        duration: f32,
        knockback: f32,
    },
    SetState {
        entity: Entity,
        state: String,
    },
    CameraShake {
        intensity: f32,
        duration: f32,
    },
    SpawnParticles {
        pos: Vec2,
        count: u32,
        color: Color,
    },
    Heal {
        entity: Entity,
        amount: i32,
    },
    Damage {
        entity: Entity,
        amount: i32,
    },
    EmitEvent {
        entity: Entity,
        name: String,
        payload: String,
    },
    Log(String),
    Purr(String),
}

/// Native Host bridge exposing Ferrium ECS queries, 2D/3D transforms, deferred commands,
/// extended math/vector intrinsics, FustyLang 0.2.1 `purr(...)` logging, JIT introspection,
/// and `meow` safety gates.
pub struct FerriumHost {
    pub snapshot: ScriptSnapshot,
    pub commands: Vec<ScriptCommand>,
    pub used_meow: bool,
    pub purr_count: u32,
}

impl FerriumHost {
    pub fn new(snapshot: ScriptSnapshot) -> Self {
        Self {
            snapshot,
            commands: Vec::new(),
            used_meow: false,
            purr_count: 0,
        }
    }

    fn expect_argc(name: &str, args: &[Value], expected: usize) -> Result<(), String> {
        if args.len() != expected {
            Err(format!(
                "'{}' expects {} argument(s), got {}",
                name,
                expected,
                args.len()
            ))
        } else {
            Ok(())
        }
    }

    fn expect_vec2(name: &str, val: &Value) -> Result<Vec2, String> {
        match val {
            Value::V2(v) => Ok(Vec2::new(v.x, v.y)),
            other => Err(format!("'{}' expected vec2, got {}", name, other)),
        }
    }

    fn expect_vec3(name: &str, val: &Value) -> Result<Vec3, String> {
        match val {
            Value::V3(v) => Ok(Vec3::new(v.x, v.y, v.z)),
            other => Err(format!("'{}' expected vec3, got {}", name, other)),
        }
    }

    fn expect_f64(name: &str, val: &Value) -> Result<f64, String> {
        val.as_f64()
            .ok_or_else(|| format!("'{}' expected number, got {}", name, val))
    }

    fn expect_i32(name: &str, val: &Value) -> Result<i32, String> {
        match val {
            Value::Int(i) => Ok(*i as i32),
            Value::Float(f) => Ok(*f as i32),
            other => Err(format!("'{}' expected integer, got {}", name, other)),
        }
    }

    fn expect_string(val: &Value) -> String {
        match val {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }
}

impl Host for FerriumHost {
    fn call_native(
        &mut self,
        name: &str,
        args: &[Value],
        meow_mode: bool,
    ) -> Result<Value, String> {
        match name {
            // --- Self & Entity Queries ---
            "get_entity_id" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Int(self.snapshot.self_entity.id as i64))
            }
            "get_self_pos" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::V2(FustyVec2::new(
                    self.snapshot.self_pos.x,
                    self.snapshot.self_pos.y,
                )))
            }
            "get_self_pos3" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::V3(FustyVec3::new(
                    self.snapshot.self_pos3.x,
                    self.snapshot.self_pos3.y,
                    self.snapshot.self_pos3.z,
                )))
            }
            "get_self_rot" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::V3(FustyVec3::new(
                    self.snapshot.self_rot.x,
                    self.snapshot.self_rot.y,
                    self.snapshot.self_rot.z,
                )))
            }
            "get_self_scale" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::V3(FustyVec3::new(
                    self.snapshot.self_scale.x,
                    self.snapshot.self_scale.y,
                    self.snapshot.self_scale.z,
                )))
            }
            "get_self_vel" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::V2(FustyVec2::new(
                    self.snapshot.self_vel.x,
                    self.snapshot.self_vel.y,
                )))
            }
            "get_self_hp" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Int(self.snapshot.self_hp as i64))
            }
            "get_self_max_hp" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Int(self.snapshot.self_max_hp as i64))
            }
            "get_hp_ratio" => {
                Self::expect_argc(name, args, 0)?;
                let max_hp = self.snapshot.self_max_hp.max(1) as f64;
                Ok(Value::Float((self.snapshot.self_hp as f64 / max_hp).clamp(0.0, 1.0)))
            }
            "is_alive" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Bool(self.snapshot.self_hp > 0))
            }
            "distance_to_player" => {
                Self::expect_argc(name, args, 0)?;
                let delta = self.snapshot.player_pos - self.snapshot.self_pos;
                Ok(Value::Float(delta.length() as f64))
            }
            "dir_to_player" => {
                Self::expect_argc(name, args, 0)?;
                let dir = (self.snapshot.player_pos - self.snapshot.self_pos).normalized();
                Ok(Value::V2(FustyVec2::new(dir.x, dir.y)))
            }
            "get_frame_count" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Int(self.snapshot.frame_count as i64))
            }
            "jit_enabled" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Bool(self.snapshot.jit_enabled))
            }
            "jit_compiled_count" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Int(self.snapshot.jit_compiled_count as i64))
            }

            // --- Transform & Visual Mutations ---
            "set_self_pos" => {
                Self::expect_argc(name, args, 1)?;
                let pos = Self::expect_vec2(name, &args[0])?;
                self.snapshot.self_pos = pos;
                self.snapshot.self_pos3.x = pos.x;
                self.snapshot.self_pos3.z = pos.y;
                self.commands
                    .push(ScriptCommand::SetPos(self.snapshot.self_entity, pos));
                Ok(Value::Nil)
            }
            "translate_self" => {
                Self::expect_argc(name, args, 1)?;
                let delta = Self::expect_vec2(name, &args[0])?;
                let pos = self.snapshot.self_pos + delta;
                self.snapshot.self_pos = pos;
                self.snapshot.self_pos3.x = pos.x;
                self.snapshot.self_pos3.z = pos.y;
                self.commands
                    .push(ScriptCommand::SetPos(self.snapshot.self_entity, pos));
                Ok(Value::V2(FustyVec2::new(pos.x, pos.y)))
            }
            "set_self_pos3" => {
                Self::expect_argc(name, args, 1)?;
                let pos = Self::expect_vec3(name, &args[0])?;
                self.snapshot.self_pos3 = pos;
                self.snapshot.self_pos = Vec2::new(pos.x, pos.z);
                self.commands
                    .push(ScriptCommand::SetPos3(self.snapshot.self_entity, pos));
                Ok(Value::Nil)
            }
            "translate_self3" => {
                Self::expect_argc(name, args, 1)?;
                let delta = Self::expect_vec3(name, &args[0])?;
                let pos = self.snapshot.self_pos3 + delta;
                self.snapshot.self_pos3 = pos;
                self.snapshot.self_pos = Vec2::new(pos.x, pos.z);
                self.commands
                    .push(ScriptCommand::SetPos3(self.snapshot.self_entity, pos));
                Ok(Value::V3(FustyVec3::new(pos.x, pos.y, pos.z)))
            }
            "set_self_rot" => {
                Self::expect_argc(name, args, 1)?;
                let rot = Self::expect_vec3(name, &args[0])?;
                self.snapshot.self_rot = rot;
                self.commands
                    .push(ScriptCommand::SetRot(self.snapshot.self_entity, rot));
                Ok(Value::Nil)
            }
            "rotate_self" => {
                Self::expect_argc(name, args, 1)?;
                let delta = Self::expect_vec3(name, &args[0])?;
                let rot = self.snapshot.self_rot + delta;
                self.snapshot.self_rot = rot;
                self.commands
                    .push(ScriptCommand::SetRot(self.snapshot.self_entity, rot));
                Ok(Value::V3(FustyVec3::new(rot.x, rot.y, rot.z)))
            }
            "set_self_scale" => {
                Self::expect_argc(name, args, 1)?;
                let scale = Self::expect_vec3(name, &args[0])?;
                self.snapshot.self_scale = scale;
                self.commands
                    .push(ScriptCommand::SetScale(self.snapshot.self_entity, scale));
                Ok(Value::Nil)
            }
            "scale_self" => {
                Self::expect_argc(name, args, 1)?;
                let factor = Self::expect_f64(name, &args[0])? as f32;
                let scale = self.snapshot.self_scale * factor;
                self.snapshot.self_scale = scale;
                self.commands
                    .push(ScriptCommand::SetScale(self.snapshot.self_entity, scale));
                Ok(Value::V3(FustyVec3::new(scale.x, scale.y, scale.z)))
            }
            "set_self_color" => {
                if args.len() != 3 && args.len() != 4 {
                    return Err(format!(
                        "'set_self_color' expects 3 or 4 arguments (r, g, b[, a]), got {}",
                        args.len()
                    ));
                }
                let r = Self::expect_i32(name, &args[0])?.clamp(0, 255) as u8;
                let g = Self::expect_i32(name, &args[1])?.clamp(0, 255) as u8;
                let b = Self::expect_i32(name, &args[2])?.clamp(0, 255) as u8;
                let a = if args.len() == 4 {
                    Self::expect_i32(name, &args[3])?.clamp(0, 255) as u8
                } else {
                    255
                };
                self.commands.push(ScriptCommand::SetColor(
                    self.snapshot.self_entity,
                    Color::rgba(r, g, b, a),
                ));
                Ok(Value::Nil)
            }
            "set_self_vel" => {
                Self::expect_argc(name, args, 1)?;
                let vel = Self::expect_vec2(name, &args[0])?;
                self.snapshot.self_vel = vel;
                self.commands
                    .push(ScriptCommand::SetVel(self.snapshot.self_entity, vel));
                Ok(Value::Nil)
            }

            // --- Player & Combat Actions ---
            "get_player_pos" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::V2(FustyVec2::new(
                    self.snapshot.player_pos.x,
                    self.snapshot.player_pos.y,
                )))
            }
            "get_player_hp" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Int(self.snapshot.player_hp as i64))
            }
            "heal_self" => {
                Self::expect_argc(name, args, 1)?;
                let amount = Self::expect_i32(name, &args[0])?.max(0);
                self.snapshot.self_hp =
                    (self.snapshot.self_hp + amount).min(self.snapshot.self_max_hp);
                self.commands.push(ScriptCommand::Heal {
                    entity: self.snapshot.self_entity,
                    amount,
                });
                Ok(Value::Int(self.snapshot.self_hp as i64))
            }
            "damage_self" => {
                Self::expect_argc(name, args, 1)?;
                let amount = Self::expect_i32(name, &args[0])?.max(0);
                self.snapshot.self_hp = (self.snapshot.self_hp - amount).max(0);
                self.commands.push(ScriptCommand::Damage {
                    entity: self.snapshot.self_entity,
                    amount,
                });
                Ok(Value::Int(self.snapshot.self_hp as i64))
            }
            "emit_event" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(format!(
                        "'emit_event' expects 1 or 2 arguments (name[, payload]), got {}",
                        args.len()
                    ));
                }
                let ev_name = Self::expect_string(&args[0]);
                let payload = if args.len() == 2 {
                    Self::expect_string(&args[1])
                } else {
                    String::new()
                };
                self.commands.push(ScriptCommand::EmitEvent {
                    entity: self.snapshot.self_entity,
                    name: ev_name,
                    payload,
                });
                Ok(Value::Nil)
            }
            "spawn_hitbox" => {
                if args.len() != 4 && args.len() != 5 {
                    return Err(format!(
                        "'spawn_hitbox' expects 4 or 5 arguments, got {}",
                        args.len()
                    ));
                }
                let offset = Self::expect_vec2(name, &args[0])?;
                let size = Self::expect_vec2(name, &args[1])?;
                let damage = Self::expect_i32(name, &args[2])?;
                let duration = Self::expect_f64(name, &args[3])? as f32;
                let knockback = if args.len() == 5 {
                    Self::expect_f64(name, &args[4])? as f32
                } else {
                    260.0
                };
                self.commands.push(ScriptCommand::SpawnHitbox {
                    owner: self.snapshot.self_entity,
                    offset,
                    size,
                    damage,
                    duration: duration.max(0.02),
                    knockback,
                });
                Ok(Value::Nil)
            }
            "set_boss_state" => {
                Self::expect_argc(name, args, 1)?;
                let state = Self::expect_string(&args[0]);
                self.commands.push(ScriptCommand::SetState {
                    entity: self.snapshot.self_entity,
                    state,
                });
                Ok(Value::Nil)
            }
            "camera_shake" => {
                Self::expect_argc(name, args, 2)?;
                let intensity = Self::expect_f64(name, &args[0])? as f32;
                let duration = Self::expect_f64(name, &args[1])? as f32;
                self.commands.push(ScriptCommand::CameraShake {
                    intensity,
                    duration,
                });
                Ok(Value::Nil)
            }
            "spawn_particles" => {
                Self::expect_argc(name, args, 5)?;
                let pos = Self::expect_vec2(name, &args[0])?;
                let count = Self::expect_i32(name, &args[1])?.clamp(1, 64) as u32;
                let r = Self::expect_i32(name, &args[2])?.clamp(0, 255) as u8;
                let g = Self::expect_i32(name, &args[3])?.clamp(0, 255) as u8;
                let b = Self::expect_i32(name, &args[4])?.clamp(0, 255) as u8;
                self.commands.push(ScriptCommand::SpawnParticles {
                    pos,
                    count,
                    color: Color::rgb(r, g, b),
                });
                Ok(Value::Nil)
            }

            // --- Logging (FustyLang 0.2.1 `log` and `purr`) ---
            "log" => {
                if args.is_empty() {
                    return Err("'log' expects at least 1 argument".into());
                }
                let text = args
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                self.commands.push(ScriptCommand::Log(text));
                Ok(Value::Nil)
            }
            "purr" => {
                if args.is_empty() {
                    return Err("'purr' expects at least 1 argument".into());
                }
                let text = args
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                self.purr_count += 1;
                self.commands.push(ScriptCommand::Purr(text));
                Ok(Value::Nil)
            }

            // --- Time & Input ---
            "dt" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Float(self.snapshot.dt))
            }
            "input_x" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Float(self.snapshot.input_x))
            }
            "input_y" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Float(self.snapshot.input_y))
            }
            "time" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Float(self.snapshot.elapsed_time))
            }

            // --- Scalar Math Intrinsics ---
            "sin" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.sin()))
            }
            "cos" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.cos()))
            }
            "tan" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.tan()))
            }
            "asin" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(
                    Self::expect_f64(name, &args[0])?.clamp(-1.0, 1.0).asin(),
                ))
            }
            "acos" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(
                    Self::expect_f64(name, &args[0])?.clamp(-1.0, 1.0).acos(),
                ))
            }
            "sqrt" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.max(0.0).sqrt()))
            }
            "atan2" => {
                Self::expect_argc(name, args, 2)?;
                let y = Self::expect_f64(name, &args[0])?;
                let x = Self::expect_f64(name, &args[1])?;
                Ok(Value::Float(y.atan2(x)))
            }
            "pow" => {
                Self::expect_argc(name, args, 2)?;
                let base = Self::expect_f64(name, &args[0])?;
                let exp = Self::expect_f64(name, &args[1])?;
                Ok(Value::Float(base.powf(exp)))
            }
            "floor" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.floor()))
            }
            "ceil" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.ceil()))
            }
            "round" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.round()))
            }
            "fract" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.fract()))
            }
            "sign" => {
                Self::expect_argc(name, args, 1)?;
                let x = Self::expect_f64(name, &args[0])?;
                let s = if x > 0.0 {
                    1.0
                } else if x < 0.0 {
                    -1.0
                } else {
                    0.0
                };
                Ok(Value::Float(s))
            }
            "deg2rad" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.to_radians()))
            }
            "rad2deg" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.to_degrees()))
            }
            "lerp" => {
                Self::expect_argc(name, args, 3)?;
                let a = Self::expect_f64(name, &args[0])?;
                let b = Self::expect_f64(name, &args[1])?;
                let t = Self::expect_f64(name, &args[2])?;
                Ok(Value::Float(a + (b - a) * t))
            }
            "smoothstep" => {
                Self::expect_argc(name, args, 3)?;
                let edge0 = Self::expect_f64(name, &args[0])?;
                let edge1 = Self::expect_f64(name, &args[1])?;
                let x = Self::expect_f64(name, &args[2])?;
                let denom = (edge1 - edge0).abs().max(1e-9);
                let t = ((x - edge0) / denom).clamp(0.0, 1.0);
                Ok(Value::Float(t * t * (3.0 - 2.0 * t)))
            }
            "abs" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.abs()))
            }
            "min" => {
                Self::expect_argc(name, args, 2)?;
                let a = Self::expect_f64(name, &args[0])?;
                let b = Self::expect_f64(name, &args[1])?;
                Ok(Value::Float(a.min(b)))
            }
            "max" => {
                Self::expect_argc(name, args, 2)?;
                let a = Self::expect_f64(name, &args[0])?;
                let b = Self::expect_f64(name, &args[1])?;
                Ok(Value::Float(a.max(b)))
            }
            "clamp" => {
                Self::expect_argc(name, args, 3)?;
                let x = Self::expect_f64(name, &args[0])?;
                let lo = Self::expect_f64(name, &args[1])?;
                let hi = Self::expect_f64(name, &args[2])?;
                Ok(Value::Float(x.clamp(lo, hi)))
            }

            // --- Vector Math Intrinsics ---
            "distance2" => {
                Self::expect_argc(name, args, 2)?;
                let a = Self::expect_vec2(name, &args[0])?;
                let b = Self::expect_vec2(name, &args[1])?;
                Ok(Value::Float((b - a).length() as f64))
            }
            "distance3" => {
                Self::expect_argc(name, args, 2)?;
                let a = Self::expect_vec3(name, &args[0])?;
                let b = Self::expect_vec3(name, &args[1])?;
                Ok(Value::Float((b - a).length() as f64))
            }
            "dot2" => {
                Self::expect_argc(name, args, 2)?;
                let a = Self::expect_vec2(name, &args[0])?;
                let b = Self::expect_vec2(name, &args[1])?;
                Ok(Value::Float((a.x * b.x + a.y * b.y) as f64))
            }
            "dot3" => {
                Self::expect_argc(name, args, 2)?;
                let a = Self::expect_vec3(name, &args[0])?;
                let b = Self::expect_vec3(name, &args[1])?;
                Ok(Value::Float((a.x * b.x + a.y * b.y + a.z * b.z) as f64))
            }
            "lerp_vec2" => {
                Self::expect_argc(name, args, 3)?;
                let a = Self::expect_vec2(name, &args[0])?;
                let b = Self::expect_vec2(name, &args[1])?;
                let t = Self::expect_f64(name, &args[2])? as f32;
                let r = a + (b - a) * t;
                Ok(Value::V2(FustyVec2::new(r.x, r.y)))
            }
            "lerp_vec3" => {
                Self::expect_argc(name, args, 3)?;
                let a = Self::expect_vec3(name, &args[0])?;
                let b = Self::expect_vec3(name, &args[1])?;
                let t = Self::expect_f64(name, &args[2])? as f32;
                let r = a + (b - a) * t;
                Ok(Value::V3(FustyVec3::new(r.x, r.y, r.z)))
            }

            // --- `meow` Safety-Gated Raw Intrinsics ---
            "raw_teleport" => {
                if !meow_mode {
                    return Err(
                        "Safety violation: raw_teleport() requires a meow { } block!".into(),
                    );
                }
                Self::expect_argc(name, args, 1)?;
                let pos = Self::expect_vec2(name, &args[0])?;
                self.snapshot.self_pos = pos;
                self.snapshot.self_pos3.x = pos.x;
                self.snapshot.self_pos3.z = pos.y;
                self.used_meow = true;
                self.commands
                    .push(ScriptCommand::RawTeleport(self.snapshot.self_entity, pos));
                Ok(Value::Nil)
            }
            "raw_teleport3" => {
                if !meow_mode {
                    return Err(
                        "Safety violation: raw_teleport3() requires a meow { } block!".into(),
                    );
                }
                Self::expect_argc(name, args, 1)?;
                let pos = Self::expect_vec3(name, &args[0])?;
                self.snapshot.self_pos3 = pos;
                self.snapshot.self_pos = Vec2::new(pos.x, pos.z);
                self.used_meow = true;
                self.commands
                    .push(ScriptCommand::RawTeleport3(self.snapshot.self_entity, pos));
                Ok(Value::Nil)
            }
            "is_meow" => {
                Self::expect_argc(name, args, 0)?;
                Ok(Value::Bool(meow_mode))
            }
            other => Err(format!("Unknown native host function '{}'", other)),
        }
    }
}
