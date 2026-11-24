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
