# FustyLang 0.2.1 Native Integration

Ferrium depends directly on [`Kirieshka14/fustylang`](https://github.com/Kirieshka14/fustylang) via Cargo `[workspace.dependencies]`:

```toml
[workspace.dependencies]
fusty_lexer    = { git = "https://github.com/Kirieshka14/fustylang.git", branch = "main" }
fusty_parser   = { git = "https://github.com/Kirieshka14/fustylang.git", branch = "main" }
fusty_bytecode = { git = "https://github.com/Kirieshka14/fustylang.git", branch = "main" }
fusty_jit      = { git = "https://github.com/Kirieshka14/fustylang.git", branch = "main" }
fusty_vm       = { git = "https://github.com/Kirieshka14/fustylang.git", branch = "main" }
```

This guarantees that every language feature, bytecode optimization, and JIT improvement in FustyLang 0.2.1 works natively inside Ferrium without glue layers or forks.

## Supported FustyLang 0.2.1 Features

| Feature | Syntax | Behavior in Ferrium |
| --- | --- | --- |
| Functions | `nya name(x: i64) -> i64 { ... }` | Compiled to bytecode functions; numeric functions are eligible for `fusty_jit` |
| Midpoint operator | `a uwu b` | Computes `(a + b) / 2` for integers (returns `f64`), floats, `vec2`, and `vec3` |
| Structured logging | `purr("HP:", hp)` | Emits `ScriptCommand::Purr` and increments `ScriptComponent::purr_count` |
| Nested block comments | `/* outer /* inner */ outer */` | Supported by both compiler and Ferrium Editor syntax highlighter |
| Coroutines | `wait 0.25;` / `yield;` | Suspends VM cleanly across frames without blocking the ECS tick |
| Immutability by default | `let x = 1;` vs `let mut x = 1;` | Enforced at bytecode compilation time |

## Safe `baka!(...)` Trapping & Recovery

FustyLang 0.2.1 introduces `baka!(msg)` for explicit runtime invariant assertions.

In Ferrium, `baka!` traps **never panic the Rust host process**:
- `FerriumScript` inspects runtime errors via `is_baka_message(&str)` and classifies them as `ScriptError::Baka`.
- `ScriptComponent` records the message in `last_baka`, increments `baka_count`, and marks the script status cleanly.
- Calling `ScriptComponent::recover_from_baka()` resets the VM instruction pointer and stack while preserving `jit_policy` and hot-reload metadata.

## `meow` Safety Gates

High-privilege engine mutations that bypass normal physics integration are explicitly gated behind FustyLang's `meow` scope:

```fusty
// Outside meow -> returns a runtime safety violation
// raw_teleport(vec2(10.0, 20.0));

// Inside meow block or `meow nya` function -> allowed!
meow {
    raw_teleport(vec2(10.0, 20.0));
    raw_teleport3(vec3(10.0, 2.0, 20.0));
}
```

Scripts can also query `is_meow()` at runtime to check whether they are currently executing inside a `meow` scope.

## Native `fusty_jit` Compilation

Ferrium integrates `fusty_jit` directly into `ScriptComponent`:

- **`JitPolicy::Auto`** *(default)*: Interprets cold code in the bytecode VM and compiles hot `nya` functions to native machine code once their call count reaches `JitPolicy::DEFAULT_HOT_THRESHOLD` (`3`).
- **`JitPolicy::Eager`**: Compiles eligible `nya` functions immediately on first invocation.
- **`JitPolicy::Disabled`**: Forces pure bytecode VM execution (useful for deterministic debugging or comparing JIT vs VM output).

### Platforms, Eligibility & Diagnostics
- **Supported Native JIT Targets**: Linux x86-64 (SysV ABI + `mmap`), Windows x86-64 (Win64 ABI + `VirtualAlloc`), and macOS x86-64 (SysV ABI + `mmap(MAP_JIT)`). Other targets automatically fall back to the bytecode VM.
- **Silent Runtime Fallback with Diagnostics**: If a function cannot be JIT-compiled (e.g., calls host intrinsics, uses `baka!`, or receives mixed argument types), execution silently falls back to the VM without interrupting the script, while recording a structured `JitFallbackDiagnostic` (`function`, `kind`, `reason`, `count`) surfaced in `JitTelemetry`, `--jit-stats`, and the Ferrium Editor toolbar tooltip.
- `compile_fusty_detailed` inspects every function in a `Chunk` using `check_jit_eligibility` and reports its `JitValueKind` in `FunctionSummary`.

## Controllable Garbage Collector (Enabled by Default)

FustyLang VMs ship with a mark-and-sweep `GcHeap` **enabled by default** alongside `FrameArena`, so scripts can allocate freely without manual memory management while still offering deterministic control for FPS-critical gameplay:

- `gc_is_enabled()` — returns `true` by default
- `gc_disable()` / `gc_enable()` — pause/resume automatic collection (e.g., during boss fights)
- `gc_collect()` — run a full mark-and-sweep cycle (e.g., on level transitions)
- `gc_step(budget)` — run a budgeted incremental GC step per frame
- `gc_live_objects()`, `gc_live_bytes()`, `gc_collections()` — inspect live heap telemetry
