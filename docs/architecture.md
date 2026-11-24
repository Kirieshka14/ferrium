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
