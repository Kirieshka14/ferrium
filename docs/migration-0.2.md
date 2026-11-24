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
