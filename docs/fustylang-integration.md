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
