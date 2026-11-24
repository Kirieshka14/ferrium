# Ferrium Studio Editor Guide

Launch the editor with:

```bash
./run.sh editor
```

## Viewport & Scene Hierarchy
- **Hybrid 2D / 3D Orbit Camera**: Drag to orbit in 3D mode or pan in 2D mode.
- **Ray3 Picking**: Clicking any 3D mesh or 2D sprite runs `Ray3::intersect_obb` against entity bounds to select it in the Inspector.
- **Scene Persistence**: Scenes serialize cleanly to JSON and round-trip all entity transforms, colors, models, and attached `.fy` scripts.

## Code Panel & Live REPL

The integrated FustyLang 0.2.1 code editor (`crates/ferrium_editor/src/syntax.rs`) highlights:
- `nya`, `let`, `mut`, `struct`, `enum`, `impl`, `true`, `false`, `self`, `nil`
- Control keywords & `meow`
- Distinct accent colors for `uwu`, `purr(...)`, and `baka!(...)`
- Properly balanced nested block comments `/* outer /* inner */ outer */`

Use the built-in **REPL** tab to evaluate one-off FustyLang expressions directly against the active `FerriumHost` snapshot.

## Bytecode & `fusty_jit` Inspector

- **Disassembly View**: Displays every compiled opcode (`Opcode::Uwu`, `Opcode::Baka`, loops, calls) alongside constant pools.
- **Per-Function JIT Badges**: Each `nya` function shows whether `fusty_jit` can compile it natively (`JIT i64` / `JIT f64`) or whether it runs in the bytecode VM.
- **Toolbar JIT Policy Switcher**: Click the JIT badge in the top toolbar to cycle `JIT: Auto` → `JIT: Eager` → `JIT: Off (VM)` live, while monitoring compiled function counts, native call hits, and emitted machine code bytes.
