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
