# Migrating Scripts to Ferrium 0.2.0 (FustyLang 0.2.1)

Ferrium 0.2.0 natively targets **FustyLang 0.2.1**. Legacy `fn` syntax is rejected by the compiler.

## Quick Syntax Checklist

| Legacy (0.1.x) | FustyLang 0.2.1 (Ferrium 0.2.0) |
| --- | --- |
| `fn update(dt: f64) { ... }` | `nya update(dt: f64) { ... }` |
| `meow fn blink() { ... }` | `meow nya blink() { ... }` |
| `(a + b) * 0.5` | `a uwu b` |
| `log("hello")` | `purr("hello")` *(or `log("hello")`)* |
| manual error return on bad state | `baka!("invariant violated")` |

## Adopting 3D Transforms & Events

Scripts that previously tracked 2D coordinates manually can now use native 3D Host API intrinsics:

```fusty
let pos3 = get_self_pos3();
set_self_pos3(pos3 + vec3(0.0, sin(time()) * 0.05, 0.0));
rotate_self(vec3(0.0, dt() * 1.5, 0.0));

if get_hp_ratio() < 0.25 {
    emit_event("low_hp", "critical");
}
```
