# Ferrium Host API Reference

All native intrinsics are exposed to FustyLang 0.2.1 scripts through `FerriumHost` (`crates/ferrium_script/src/host.rs`).

## Entity, Health & Player Queries

| Intrinsic | Signature | Description |
| --- | --- | --- |
| `get_entity_id()` | `() -> i64` | Current ECS entity ID |
| `get_self_hp()` | `() -> i64` | Current entity HP |
| `get_self_max_hp()` | `() -> i64` | Maximum entity HP |
| `get_hp_ratio()` | `() -> f64` | Normalized HP in `0.0..=1.0` |
| `is_alive()` | `() -> bool` | `true` when `self_hp > 0` |
| `get_player_pos()` | `() -> vec2` | Player 2D world position |
| `get_player_hp()` | `() -> i64` | Player current HP |
| `distance_to_player()` | `() -> f64` | Euclidean 2D distance from self to player |
| `dir_to_player()` | `() -> vec2` | Normalized 2D direction vector toward player |

## 2D/3D Transforms, Velocity & Visuals

| Intrinsic | Signature | Description |
| --- | --- | --- |
| `get_self_pos()` | `() -> vec2` | Current 2D position |
| `set_self_pos(pos)` | `(vec2) -> nil` | Emits `ScriptCommand::SetPos` |
| `translate_self(delta)` | `(vec2) -> vec2` | Translates 2D position and returns new `vec2` |
| `get_self_pos3()` | `() -> vec3` | Current 3D position |
| `set_self_pos3(pos)` | `(vec3) -> nil` | Emits `ScriptCommand::SetPos3` |
| `translate_self3(delta)` | `(vec3) -> vec3` | Translates 3D position and returns new `vec3` |
| `get_self_rot()` | `() -> vec3` | Current Euler XYZ rotation |
| `set_self_rot(rot)` | `(vec3) -> nil` | Emits `ScriptCommand::SetRot` |
| `rotate_self(delta)` | `(vec3) -> vec3` | Adds Euler rotation delta and returns new `vec3` |
| `get_self_scale()` | `() -> vec3` | Current 3D scale |
| `set_self_scale(scale)` | `(vec3) -> nil` | Emits `ScriptCommand::SetScale` |
| `scale_self(factor)` | `(f64) -> vec3` | Multiplies scale uniformly and returns new `vec3` |
| `get_self_vel()` | `() -> vec2` | Current 2D velocity |
| `set_self_vel(vel)` | `(vec2) -> nil` | Emits `ScriptCommand::SetVel` |
| `set_self_color(r, g, b[, a])` | `(i64, i64, i64[, i64]) -> nil` | Clamps channels to `0..=255` and emits `ScriptCommand::SetColor` |

## Combat, Effects & Events

| Intrinsic | Signature | Description |
| --- | --- | --- |
| `heal_self(amount)` | `(i64) -> i64` | Clamps HP up to `max_hp`, emits `Heal`, returns new HP |
| `damage_self(amount)` | `(i64) -> i64` | Clamps HP down to `0`, emits `Damage`, returns new HP |
| `spawn_hitbox(offset, size, dmg, dur[, kb])` | `(vec2, vec2, i64, f64[, f64]) -> nil` | Spawns combat hitbox (default knockback `260.0`) |
| `set_boss_state(name)` | `(string) -> nil` | Updates boss state label |
| `camera_shake(intensity, duration)` | `(f64, f64) -> nil` | Triggers screen shake effect |
| `spawn_particles(pos, count, r, g, b)` | `(vec2, i64, i64, i64, i64) -> nil` | Spawns `1..=64` colored particles |
| `emit_event(name[, payload])` | `(string[, string]) -> nil` | Emits `ScriptCommand::EmitEvent` |

## Scalar & Vector Math Intrinsics

| Intrinsic | Signature | Description |
| --- | --- | --- |
| `sin(x)`, `cos(x)`, `tan(x)` | `(f64) -> f64` | Trigonometric functions (radians) |
| `asin(x)`, `acos(x)` | `(f64) -> f64` | Clamped inverse sine / cosine |
| `atan2(y, x)` | `(f64, f64) -> f64` | Four-quadrant arctangent |
| `sqrt(x)`, `pow(base, exp)`, `abs(x)` | `(...) -> f64` | Square root (non-negative clamped), power, absolute value |
| `floor(x)`, `ceil(x)`, `round(x)`, `fract(x)`, `sign(x)` | `(f64) -> f64` | Rounding, fractional part, and sign (`-1.0`/`0.0`/`1.0`) |
| `min(a, b)`, `max(a, b)`, `clamp(x, lo, hi)` | `(...) -> f64` | Numeric bounds |
| `deg2rad(deg)`, `rad2deg(rad)` | `(f64) -> f64` | Angle unit conversions |
| `lerp(a, b, t)`, `smoothstep(e0, e1, x)` | `(f64, f64, f64) -> f64` | Linear and Hermite interpolation |
| `distance2(a, b)`, `distance3(a, b)` | `(vec, vec) -> f64` | 2D / 3D Euclidean distance |
| `dot2(a, b)`, `dot3(a, b)` | `(vec, vec) -> f64` | 2D / 3D dot product |
| `lerp_vec2(a, b, t)`, `lerp_vec3(a, b, t)` | `(vec, vec, f64) -> vec` | Vector linear interpolation |

## Time, Input, Logging, JIT & `meow` Intrinsics

| Intrinsic | Signature | Description |
| --- | --- | --- |
| `dt()` | `() -> f64` | Frame delta time in seconds |
| `time()` | `() -> f64` | Total elapsed simulation time |
| `get_frame_count()` | `() -> i64` | Total simulated frames |
| `input_x()`, `input_y()` | `() -> f64` | Normalized player movement axes |
| `log(a, ...)` | `(any...) -> nil` | Emits `ScriptCommand::Log` |
| `purr(a, ...)` | `(any...) -> nil` | FustyLang 0.2.1 structured log (`ScriptCommand::Purr`) |
| `jit_enabled()` | `() -> bool` | Whether native `fusty_jit` is active |
| `jit_compiled_count()` | `() -> i64` | Count of JIT-compiled functions |
| `is_meow()` | `() -> bool` | `true` inside a `meow` block or `meow nya` function |
| `raw_teleport(pos)` | `(vec2) -> nil` | **Requires `meow`**: instant 2D teleport |
| `raw_teleport3(pos)` | `(vec3) -> nil` | **Requires `meow`**: instant 3D teleport |
