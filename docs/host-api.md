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
