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
