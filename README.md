# Ferrium Engine

[![CI](https://github.com/Kirieshka14/ferrium/actions/workflows/ci.yml/badge.svg)](https://github.com/Kirieshka14/ferrium/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![FustyLang](https://img.shields.io/badge/FustyLang-0.2.1%20Native-ff69b4.svg)](https://github.com/Kirieshka14/fustylang)

**Ferrium** is a modular 2D/3D Rust game engine and visual studio IDE built around **native [FustyLang 0.2.1](https://github.com/Kirieshka14/fustylang) scripting**, featuring a generational SparseSet ECS, a hybrid 2D/3D software renderer, and zero-overhead `fusty_jit` native machine code compilation.

## Highlights

- **Native FustyLang 0.2.1 Integration**: Direct Git dependency on `Kirieshka14/fustylang` (`fusty_lexer`, `fusty_parser`, `fusty_bytecode`, `fusty_vm`, `fusty_jit`) — zero forks, zero shims.
- **Full 0.2.1 Language Support**: `nya` functions, `uwu` midpoint operator (scalars, `vec2`, `vec3`), `purr(...)` structured logging, nested block comments `/* /* */ */`, and safe `baka!(...)` trap catching & recovery.
- **Native `fusty_jit` Execution**: Hot numeric `nya` functions compile to native machine code at runtime with configurable `Auto`, `Eager`, and `Disabled` policies and live telemetry.
- **Safety-First Gameplay Scripting**: `meow { ... }` blocks explicitly gate raw engine teleportation intrinsics, while per-frame step budgets prevent infinite loops from freezing the engine.
- **Hybrid 2D/3D Software Renderer**: Zero-GPU-driver-headache software rasterizer with 3D OBJ mesh loading, orbit camera, 2D sprites, and HUD text.
- **Ferrium Studio IDE**: Live 2D/3D viewport, gizmo picking, FustyLang 0.2.1 syntax highlighter, live REPL, bytecode disassembler, per-function JIT eligibility badges, and hot-reload.

## Workspace Architecture

| Crate | Description |
| --- | --- |
| [`ferrium_math`](crates/ferrium_math) | `Vec2`, `Vec3`, `Vec4`, `Mat3`, `Mat4`, `Ray3`, `Rect`, `Transform`, and divide-free RGBA `Color` blending |
| [`ferrium_ecs`](crates/ferrium_ecs) | Generational `Entity` allocator, `SparseSet<T>` component pools, singleton resources, and 5-stage `Schedule` |
| [`ferrium_render`](crates/ferrium_render) | `SoftwareCanvas`, `RenderQueue`, `Camera` (2D / 2.5D / 3D orbit), Wavefront `.obj` `Mesh3D`, and `Sprite2D` |
| [`ferrium_script`](crates/ferrium_script) | Native FustyLang 0.2.1 compiler, `ScriptComponent`, `FerriumHost` API bridge, `JitPolicy`, and `JitTelemetry` |
| [`ferrium_engine`](crates/ferrium_engine) | Standalone playable runtime & deterministic headless verification CLI (`--headless-ticks`, `--jit-mode`, `--jit-stats`) |
| [`ferrium_editor`](crates/ferrium_editor) | Visual `egui`/`eframe` studio editor with live viewport, inspector, code panel, and bytecode/JIT debugger |

## Quick Start

### Prerequisites
- Rust stable toolchain (`rustc` / `cargo`)
- On Linux: standard windowing dev libs (`libxkbcommon-dev`, `libxcb-*`, `libgtk-3-dev`)

### Run Studio Editor or Standalone Engine

```bash
git clone https://github.com/Kirieshka14/ferrium.git
cd ferrium

# Launch Ferrium Studio Editor
./run.sh editor

# Launch standalone Boss Arena runtime
./run.sh game

# Run all workspace unit & integration tests
./run.sh test

# Run deterministic headless verification for Editor + Engine
./run.sh headless
```

## FustyLang 0.2.1 Scripting Example

```fusty
/* Native FustyLang 0.2.1 script running inside Ferrium ECS */
nya mitigate(raw_dmg: i64, armor: i64) -> i64 {
    let reduced = raw_dmg - armor;
    if reduced < 1 {
        return 1;
    }
    return reduced;
}

let target = get_player_pos();
let current = get_self_pos();

// Smooth midpoint approach using the `uwu` operator
set_self_pos(current uwu target);

// Safe runtime invariant guard (caught by Ferrium without crashing)
let dmg = mitigate(35, 10);
if dmg <= 0 {
    baka!("damage must remain positive");
}

// Explicit `meow` block required for raw teleportation
if get_hp_ratio() < 0.5 {
    meow {
        raw_teleport(vec2(0.0, 0.0));
    }
    purr("Phase 2 blink executed!");
}
```

## Native `fusty_jit` & Engine CLI

`ferrium_engine` exposes full control over FustyLang's JIT compiler from the command line:

```bash
# Default Auto policy (compiles hot `nya` functions after 3 calls)
cargo run -p ferrium_engine -- --headless-ticks 120 --jit-stats

# Eager policy (compiles eligible `nya` functions on first call)
cargo run -p ferrium_engine -- --headless-ticks 120 --jit-mode eager --jit-stats

# Pure bytecode VM interpreter (JIT disabled)
cargo run -p ferrium_engine -- --headless-ticks 120 --no-jit --jit-stats

# Run a custom script
cargo run -p ferrium_engine -- --script examples/jit_benchmark.fy --headless-ticks 30 --jit-stats
```

## Ferrium Studio Editor

Launch the visual editor with `./run.sh editor` (or verify it headlessly via `cargo run -p ferrium_editor -- --headless-check`):

- **2D / 3D Orbit Viewport**: Toggle between 2D/2.5D and 3D orbit projection, pick entities via `Ray3` OBB intersection, and manipulate transforms with gizmos.
- **FustyLang 0.2.1 Code Panel**: Dedicated syntax highlighting for `nya`, `meow`, `uwu`, `purr`, `baka!`, and nested block comments, plus one-click starter script templates.
- **Bytecode & JIT Inspector**: Live disassembly of compiled FustyLang bytecode (`Opcode::Uwu`, `Opcode::Baka`, etc.), per-function JIT eligibility badges, and a toolbar JIT policy switcher (`Auto` / `Eager` / `Off`) with live native call counters.
- **Resilient Hot-Reload**: Editing a `.fy` script reloads automatically; syntax errors or `baka!` traps are surfaced cleanly without tearing down the scene.
