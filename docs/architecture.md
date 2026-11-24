# Ferrium Engine Architecture

Ferrium is structured as a six-crate Cargo workspace with strict unidirectional layering:

```
           ┌────────────────┐   ┌────────────────┐
           │ ferrium_engine │   │ ferrium_editor │
           └───────┬────────┘   └───────┬────────┘
                   │                    │
         ┌─────────┴──────────┬─────────┴────────┐
         ▼                    ▼                  ▼
  ┌──────────────┐   ┌────────────────┐  ┌──────────────┐
  │ferrium_render│   │ ferrium_script │  │ fusty_* 0.2.1│
  └──────┬───────┘   └───────┬────────┘  └──────────────┘
         │                   │
         ▼                   ▼
  ┌──────────────┐   ┌──────────────┐
  │ ferrium_math │   │ ferrium_ecs  │
  └──────────────┘   └──────────────┘
```

### Core Principles
1. **Deterministic & Headless-First**: Both `ferrium_engine` and `ferrium_editor` expose headless verification modes for CI and automated testing.
2. **Zero External C/GPU Runtime Fragility**: `SoftwareCanvas` renders directly to a CPU framebuffer, ensuring pixel-identical output across Linux, macOS, Windows, and headless containers.
3. **Native Language Integration**: FustyLang 0.2.1 is compiled and executed in-process with zero FFI marshaling overhead.

## Entity-Component-System (`ferrium_ecs`)

- **Generational Handles**: `Entity { id: u32, generation: u32 }` detects stale references in O(1) time. Despawning increments the slot's generation and pushes the ID onto a reuse freelist.
- **Dense `SparseSet<T>` Pools**: Components are stored contiguously for cache-friendly iteration, with O(1) lookup, insertion, and swap-remove deletion.
- **Singleton Resources**: Global frame state (input, telemetry, timers) is stored by `TypeId` on `World`.
- **Deterministic 5-Stage `Schedule`**:
  1. `Stage::Input`
  2. `Stage::Script`
  3. `Stage::FixedUpdate`
  4. `Stage::Combat`
  5. `Stage::Cleanup`

## Hybrid 2D/3D Software Renderer (`ferrium_render`)

`RenderQueue` decouples simulation from rasterization:
- Systems push depth-sorted `WorldCommand` items (2D rects, polygons, sprites, 3D wire/solid boxes, `Mesh3D` instances, gizmos, fading ground grids) and `HudCommand` overlays.
- `SoftwareCanvas::render_frame` sorts world commands by Z-order, projects 3D geometry through `Camera` (`Orthographic2D`, `Hybrid25D`, or `use_orbit_3d`), and rasterizes with divide-free alpha blending (`Color::blend_over`).
- `Mesh3D::from_obj_str` parses Wavefront `.obj` files (including quads, n-gons, negative indices, and `v/vt/vn` triples) and normalizes bounds automatically.

## Native FustyLang Execution Pipeline (`ferrium_script`)

Each scripted entity owns a `ScriptComponent`:
1. **Snapshot Phase**: The engine builds an immutable `ScriptSnapshot` containing the entity's 2D/3D transform, velocity, HP, player state, input axes, `dt`, elapsed time, frame counter, and JIT telemetry.
2. **Execution Phase**: `ScriptComponent::step` runs the FustyLang 0.2.1 VM (with `fusty_jit` acceleration for hot `nya` functions) against a `FerriumHost` instance.
3. **Deferred Command Flush**: Host mutations emit typed `ScriptCommand` values (`SetPos`, `SetPos3`, `SetRot`, `SetScale`, `SetColor`, `SetVel`, `SpawnHitbox`, `Heal`, `Damage`, `EmitEvent`, `RawTeleport`, `RawTeleport3`, `Purr`, `Log`) which the engine applies atomically to `World` after the VM yields.

## Resilience & Safety Model

- **Step-Budget Guard**: Every VM tick executes under a bounded instruction budget so accidental infinite loops in `.fy` scripts return a recoverable `ScriptError::Runtime` instead of hanging the main loop.
- **Safe `baka!` Trapping**: Calling `baka!("reason")` halts only the offending script tick, increments `baka_count`, records `last_baka`, and allows clean recovery via `recover_from_baka()` while keeping the configured `JitPolicy`.
- **Live Hot-Reload**: File modification timestamps are polled automatically. If an edited script contains a lexer, parser, or bytecode error, the previous healthy VM state remains intact while the error is displayed in the console/HUD.
