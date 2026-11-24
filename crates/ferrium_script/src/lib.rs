use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use ferrium_ecs::Entity;
use ferrium_math::{Color, Vec2, Vec3};
use fusty_bytecode::chunk::Chunk;
use fusty_bytecode::compiler::Compiler;
use fusty_lexer::Lexer;
use fusty_parser::Parser;
use fusty_vm::value::{Value, Vec2 as FustyVec2, Vec3 as FustyVec3};
use fusty_vm::vm::{Host, Vm};
pub use fusty_vm::vm::VmResult;

/// Compiles FustyLang source code through Lexer -> Parser -> Bytecode Compiler.
pub fn compile_fusty_source(source: &str) -> Result<Chunk, String> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|(msg, span)| format!("Lexer {}:{}: {}", span.line, span.col, msg))?;
    let program = Parser::new(tokens)
        .parse_program()
        .map_err(|e| format!("Parser {}:{}: {}", e.span.line, e.span.col, e.message))?;
    Compiler::new()
        .compile_program(&program)
        .map_err(|e| format!("Compile error: {}", e.message))
}

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
}

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
    Log(String),
}

/// ECS component holding a compiled FustyLang script VM, coroutine timer, and hot-reload state.
pub struct ScriptComponent {
    pub path: PathBuf,
    pub chunk: Chunk,
    pub vm: Vm,
    pub wait_remaining: f64,
    pub last_mtime: Option<SystemTime>,
    pub poll_timer: f64,
    pub reload_count: u32,
    pub last_error: Option<String>,
    pub status: String,
    pub meow_flash_timer: f32,
}

impl ScriptComponent {
    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let source = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read '{}': {}", path.display(), e))?;
        let mtime = fs::metadata(&path).and_then(|m| m.modified()).ok();
        let mut comp = Self::from_source(path, &source)?;
        comp.last_mtime = mtime;
        Ok(comp)
    }

    pub fn from_source(path: impl Into<PathBuf>, source: &str) -> Result<Self, String> {
        let chunk = compile_fusty_source(source)?;
        Ok(Self {
            path: path.into(),
            chunk,
            vm: Vm::new(),
            wait_remaining: 0.0,
            last_mtime: None,
            poll_timer: 0.0,
            reload_count: 0,
            last_error: None,
            status: "ready".into(),
            meow_flash_timer: 0.0,
        })
    }

    /// Checks file mtime and hot-reloads bytecode on change while preserving the last valid script on error.
    pub fn try_hot_reload(&mut self, force: bool) -> Option<Result<u32, String>> {
        let mtime = fs::metadata(&self.path).and_then(|m| m.modified()).ok();
        let changed = match (self.last_mtime, mtime) {
            (Some(prev), Some(curr)) => curr != prev,
            (None, Some(_)) => true,
            _ => false,
        };
        if !force && !changed {
            return None;
        }
        if let Some(m) = mtime {
            self.last_mtime = Some(m);
        }

        let source = match fs::read_to_string(&self.path) {
            Ok(s) => s,
            Err(e) => {
                let msg = format!("Read error: {}", e);
                self.last_error = Some(msg.clone());
                return Some(Err(msg));
            }
        };

        match compile_fusty_source(&source) {
            Ok(new_chunk) => {
                self.chunk = new_chunk;
                self.vm.reset_execution();
                self.wait_remaining = 0.0;
                self.reload_count += 1;
                self.last_error = None;
                self.status = format!("reloaded #{}", self.reload_count);
                Some(Ok(self.reload_count))
            }
            Err(err) => {
                self.last_error = Some(err.clone());
                Some(Err(err))
            }
        }
    }
}

/// Host bridge exposing Ferrium ECS queries, 2D/3D model transforms, deferred mutation commands, math, and `meow` gates.
pub struct FerriumHost {
    pub snapshot: ScriptSnapshot,
    pub commands: Vec<ScriptCommand>,
    pub used_meow: bool,
}

impl FerriumHost {
    pub fn new(snapshot: ScriptSnapshot) -> Self {
        Self {
            snapshot,
            commands: Vec::new(),
            used_meow: false,
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
}

impl Host for FerriumHost {
    fn call_native(
        &mut self,
        name: &str,
        args: &[Value],
        meow_mode: bool,
    ) -> Result<Value, String> {
        match name {
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
            "set_self_pos3" => {
                Self::expect_argc(name, args, 1)?;
                let pos = Self::expect_vec3(name, &args[0])?;
                self.snapshot.self_pos3 = pos;
                self.snapshot.self_pos = Vec2::new(pos.x, pos.z);
                self.commands
                    .push(ScriptCommand::SetPos3(self.snapshot.self_entity, pos));
                Ok(Value::Nil)
            }
            "set_self_rot" => {
                Self::expect_argc(name, args, 1)?;
                let rot = Self::expect_vec3(name, &args[0])?;
                self.snapshot.self_rot = rot;
                self.commands
                    .push(ScriptCommand::SetRot(self.snapshot.self_entity, rot));
                Ok(Value::Nil)
            }
            "set_self_scale" => {
                Self::expect_argc(name, args, 1)?;
                let scale = Self::expect_vec3(name, &args[0])?;
                self.snapshot.self_scale = scale;
                self.commands
                    .push(ScriptCommand::SetScale(self.snapshot.self_entity, scale));
                Ok(Value::Nil)
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
                let state = match &args[0] {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
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
            "sin" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.sin()))
            }
            "cos" => {
                Self::expect_argc(name, args, 1)?;
                Ok(Value::Float(Self::expect_f64(name, &args[0])?.cos()))
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
            "lerp" => {
                Self::expect_argc(name, args, 3)?;
                let a = Self::expect_f64(name, &args[0])?;
                let b = Self::expect_f64(name, &args[1])?;
                let t = Self::expect_f64(name, &args[2])?;
                Ok(Value::Float(a + (b - a) * t))
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

#[cfg(test)]
mod tests {
    use super::*;
    use fusty_vm::vm::VmResult;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_snapshot() -> ScriptSnapshot {
        ScriptSnapshot {
            self_entity: Entity {
                id: 1,
                generation: 0,
            },
            self_pos: Vec2::new(200.0, 0.0),
            self_pos3: Vec3::new(200.0, 0.0, 0.0),
            self_rot: Vec3::ZERO,
            self_scale: Vec3::ONE,
            self_vel: Vec2::ZERO,
            self_hp: 1000,
            self_max_hp: 1000,
            player_pos: Vec2::new(-100.0, 0.0),
            player_hp: 100,
            input_x: 0.0,
            input_y: 0.0,
            dt: 1.0 / 60.0,
            elapsed_time: 1.5,
        }
    }

    #[test]
    fn test_script_coroutine_and_commands() {
        let src = r#"
            set_boss_state("windup");
            wait 0.25;
            let dir = (get_player_pos() - get_self_pos()).normalized();
            set_self_vel(dir * 300.0);
            spawn_hitbox(dir * 35.0, vec2(60.0, 60.0), 20, 0.2);
            log("strike", 1);
        "#;
        let mut comp = ScriptComponent::from_source("inline.fy", src).unwrap();
        let mut host = FerriumHost::new(test_snapshot());

        match comp.vm.run_with_host(&comp.chunk, &mut host).unwrap() {
            VmResult::Waiting { duration_sec } => assert!((duration_sec - 0.25).abs() < 1e-6),
            other => panic!("Expected wait, got {:?}", other),
        }
        assert!(matches!(
            host.commands.as_slice(),
            [ScriptCommand::SetState { state, .. }] if state == "windup"
        ));

        let mut host2 = FerriumHost::new(test_snapshot());
        assert!(matches!(
            comp.vm.run_with_host(&comp.chunk, &mut host2).unwrap(),
            VmResult::Done(_)
        ));
        assert!(host2
            .commands
            .iter()
            .any(|c| matches!(c, ScriptCommand::SpawnHitbox { damage: 20, .. })));
        assert!(host2
            .commands
            .iter()
            .any(|c| matches!(c, ScriptCommand::Log(m) if m == "strike 1")));
    }

    #[test]
    fn test_meow_teleport_gate() {
        let mut comp_safe =
            ScriptComponent::from_source("bad.fy", "raw_teleport(vec2(10.0, 20.0));").unwrap();
        let mut host = FerriumHost::new(test_snapshot());
        assert!(comp_safe
            .vm
            .run_with_host(&comp_safe.chunk, &mut host)
            .is_err());

        let mut comp_meow = ScriptComponent::from_source(
            "ok.fy",
            "meow { raw_teleport(vec2(10.0, 20.0)); }",
        )
        .unwrap();
        let mut host2 = FerriumHost::new(test_snapshot());
        comp_meow.vm.run_with_host(&comp_meow.chunk, &mut host2).unwrap();
        assert!(host2.used_meow);
        assert!(host2
            .commands
            .iter()
            .any(|c| matches!(c, ScriptCommand::RawTeleport(_, p) if *p == Vec2::new(10.0, 20.0))));
    }

    #[test]
    fn test_hot_reload_resilience_on_syntax_error() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_path = std::env::temp_dir().join(format!("ferrium_hot_reload_{}.fy", nanos));
        fs::write(&tmp_path, "set_boss_state(\"idle\");").unwrap();

        let mut comp = ScriptComponent::from_file(&tmp_path).unwrap();
        let orig_len = comp.chunk.code.len();

        // Broken update must fail reload and preserve working chunk
        fs::write(&tmp_path, "let broken = ;;;").unwrap();
        let err_res = comp.try_hot_reload(true).unwrap();
        assert!(err_res.is_err());
        assert!(comp.last_error.is_some());
        assert_eq!(comp.chunk.code.len(), orig_len);

        // Valid update succeeds and bumps reload_count
        fs::write(&tmp_path, "set_boss_state(\"enrage\"); wait 0.5;").unwrap();
        let ok_res = comp.try_hot_reload(true).unwrap();
        assert_eq!(ok_res.unwrap(), 1);
        assert!(comp.last_error.is_none());

        let _ = fs::remove_file(&tmp_path);
    }
}
