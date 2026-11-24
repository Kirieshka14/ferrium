# Changelog

All notable changes to **Ferrium Engine** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-19

### Added
- Initial Rust workspace with `ferrium_math`, `ferrium_ecs`, `ferrium_render`, `ferrium_script`, `ferrium_engine`, and `ferrium_editor`.
- Generational `SparseSet` ECS and 5-stage deterministic `Schedule`.
- Hybrid 2D/3D `SoftwareCanvas` renderer with Wavefront `.obj` mesh loader and orbit camera.
- Initial embedded scripting prototype and Boss Arena demo.

## [0.2.0] - 2026-11-24

### Added
- **Native FustyLang 0.2.1 Integration**: Replaced legacy embedded `fusty_*` crates with direct workspace Git dependencies on [`Kirieshka14/fustylang`](https://github.com/Kirieshka14/fustylang).
- **Native `fusty_jit` Support**: Added `JitPolicy` (`Auto`, `Eager`, `Disabled`), `JitTelemetry`, per-function `check_jit_eligibility`, CLI flags (`--jit`, `--no-jit`, `--jit-mode`, `--jit-stats`), and Editor toolbar policy toggle + telemetry readout.
- **Extended Host API**: Added 3D transform mutations (`get_self_pos3`, `set_self_pos3`, `translate_self3`, `set_self_rot`, `rotate_self`, `set_self_scale`, `scale_self`), color/health/event commands (`set_self_color`, `heal_self`, `damage_self`, `emit_event`), `purr(...)`, scalar & vector math intrinsics, JIT introspection, and `meow`-gated `raw_teleport3`.
- **Safe `baka!(...)` Trap Recovery**: `ScriptComponent` catches FustyLang `baka!` traps cleanly without crashing the engine or editor, preserving JIT policy on recovery.
- **Editor 0.2.1 Upgrades**: Updated `syntax.rs` for `nya`, `uwu`, `purr`, `baka!`, and nested block comments `/* /* */ */`; updated bytecode disassembler and script templates.
- **Showcase Scripts & Tests**: Migrated all bundled `.fy` scripts to FustyLang 0.2.1, added 11 showcase examples, and expanded the workspace test suite to 150+ unit and integration tests.
- **Documentation & CI**: Added full English documentation (`README.md`, `docs/*`), GitHub templates, and multi-mode headless JIT verification in CI.

### Changed
- Unified all 6 workspace crates under version `0.2.0` with shared workspace metadata.
