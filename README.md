# Ferrium

**Ferrium** is a lightweight 2D/3D Rust game engine and visual studio editor with **native [FustyLang 0.2.1](https://github.com/Kirieshka14/fustylang) scripting** and tiered x86-64 JIT execution.

- Native FustyLang 0.2.1 (`nya`, `uwu`, `baka!`, `purr`, `meow`, coroutines) directly from `Kirieshka14/fustylang`
- Tiered `fusty_jit` (`Auto`, `Eager`, `Disabled`) with live telemetry and safe `baka!` trap recovery
- Generational `SparseSet` ECS and hybrid 2D/3D software renderer (`.obj` meshes, orbit camera, sprites)
- Visual Studio IDE (`egui`) with syntax highlighting, live REPL, bytecode disassembler, and hot-reload

## Table of Contents

- [Quick Start](#quick-start)
- [Scripting Tutorial](#scripting-tutorial)
- [What's New](#whats-new)
- [Architecture](#architecture)
- [Docs](#docs)
- [Roadmap](#roadmap)
- [License](#license)

## Quick Start

```bash
# Launch Ferrium Studio Editor
./run.sh editor

# Launch standalone Boss Arena runtime
./run.sh game

# Run headless verification with JIT stats
cargo run -p ferrium_engine -- --headless-ticks 120 --jit-stats

# Force eager JIT or pure VM interpreter
cargo run -p ferrium_engine -- --script examples/jit_benchmark.fy --headless-ticks 20 --jit-mode eager --jit-stats
cargo run -p ferrium_engine -- --headless-ticks 60 --no-jit --jit-stats

# Run all workspace tests
cargo test --workspace
```

## Scripting Tutorial

### Functions & JIT: `nya`

Pure numeric `nya` functions are automatically eligible for native `fusty_jit` compilation.

```rust
nya mitigate(raw_dmg: i64, armor: i64) -> i64 {
    let reduced = raw_dmg - armor;
    if reduced < 1 { return 1; }
    return reduced;
}
```

### Smooth midpoint: `uwu`

```rust
let next_pos = get_self_pos() uwu get_player_pos();
set_self_pos(next_pos);
```

### Safe traps: `baka!`

`baka!` halts only the current script tick without crashing the engine or editor.

```rust
if dmg <= 0 {
    baka!("damage must remain positive");
}
```

### Safety-gated raw ops: `meow`

Direct teleports require an explicit `meow` block (or `meow nya` function).

```rust
meow {
    raw_teleport(vec2(0.0, 0.0));
}
```

### Coroutines & logging: `wait`, `yield`, `purr`

```rust
set_boss_state("windup");
wait 0.25;
purr("HP ratio =", get_hp_ratio());
yield;
```

### Common Host API

| Function | Description |
| :--- | :--- |
| `get_self_pos()` / `set_self_pos(v)` | 2D entity position (`get_self_pos3` / `set_self_pos3` for 3D) |
| `rotate_self(v)` / `scale_self(f)` | 3D Euler rotation and uniform scaling |
| `set_self_color(r, g, b[, a])` | Tint entity color (0..=255) |
| `heal_self(n)` / `damage_self(n)` | Modify entity HP and emit combat commands |
| `spawn_hitbox(...)` / `camera_shake(...)` | Spawn hitboxes and trigger screen shake |
| `emit_event(name[, payload])` | Dispatch gameplay events |
| `purr(...)` / `log(...)` | Structured logger / standard log |
| `raw_teleport(v)` / `raw_teleport3(v)` | Instant teleport (requires `meow`) |

See [`docs/host-api.md`](docs/host-api.md) for the full list, and [`examples/`](examples) / [`scripts/`](scripts) for playable `.fy` scripts.

## What's New

### 0.2.0

- Native **FustyLang 0.2.1** integration via direct Git workspace dependencies (`fusty_lexer`, `fusty_parser`, `fusty_bytecode`, `fusty_jit`, `fusty_vm`).
- Native `fusty_jit` policies (`Auto`, `Eager`, `Disabled`), CLI flags (`--jit`, `--no-jit`, `--jit-mode`, `--jit-stats`), and Editor toolbar telemetry.
- Safe `baka!` trap recovery, `purr(...)`, 3D transform & event Host APIs, and updated 0.2.1 syntax highlighting.
- 151 unit & integration tests across all 6 crates.

See [CHANGELOG.md](CHANGELOG.md) for the full history.

## Architecture

| Crate | Purpose |
| :--- | :--- |
| `crates/ferrium_math` | `Vec2`, `Vec3`, `Mat3`, `Mat4`, `Ray3`, `Rect`, `Transform`, divide-free RGBA `Color` |
| `crates/ferrium_ecs` | Generational `Entity`, `SparseSet<T>` pools, resources, 5-stage `Schedule` |
| `crates/ferrium_render` | Hybrid 2D/3D `SoftwareCanvas`, `Camera`, `.obj` `Mesh3D`, `Sprite2D`, `RenderQueue` |
| `crates/ferrium_script` | FustyLang 0.2.1 compiler, `ScriptComponent`, `FerriumHost`, `JitPolicy`, `JitTelemetry` |
| `crates/ferrium_engine` | Playable runtime & deterministic headless verification CLI |
| `crates/ferrium_editor` | Visual `egui` studio IDE with viewport, syntax highlighter, REPL, and JIT inspector |

## Docs

- [Architecture](docs/architecture.md)
- [FustyLang 0.2.1 Integration](docs/fustylang-integration.md)
- [Host API Reference](docs/host-api.md)
- [Editor Guide](docs/editor-guide.md)
- [Migration Guide (0.1 → 0.2)](docs/migration-0.2.md)
- [Specification](SPEC.md)

## Roadmap

- [ ] Audio mixer & spatial sound Host API
- [ ] Tilemap & 2D physics broadphase
- [ ] Multi-script entity messaging & shared blackboard resources
- [ ] Optional `wgpu` hardware-accelerated render backend

## License

[MIT](LICENSE)
