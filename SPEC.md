# FustyLang (`.fy`) Language Specification

> Concise, ground-truth reference for the FustyLang compiler, bytecode VM, and Ferrium Engine host API.
> Derived directly from `fusty_lexer`, `fusty_parser`, `fusty_bytecode`, `fusty_vm`, and `ferrium_script`.

---

## 1. Lexical Structure

### Comments & Whitespace
- Whitespace: `' '`, `'\t'`, `'\r'`, `'\n'`
- Single-line comment: `// ...`
- Multi-line comment: `/* ... */` (**supports nesting**; unterminated is a lexer error)

### Identifiers & Keywords
- Identifiers: `[a-zA-Z_][a-zA-Z0-9_]*`
- Active keywords:
  `fn`, `let`, `mut`, `if`, `else`, `match`, `while`, `return`, `break`, `continue`, `meow`, `wait`, `yield`, `true`, `false`, `vec2`, `vec3`, `_`
- Parsed-only keywords (AST only, no-op in compiler):
  `struct`, `enum`, `impl`, `self`
- Reserved / lexed-only keywords (not parsed yet):
  `for`, `in`, `spawn`, `Self`

### Literals
| Kind | Examples | Runtime Type | Notes |
|------|----------|--------------|-------|
| Integer | `42`, `1_000_000` | `Int(i64)` | Underscores `_` allowed as separators |
| Float | `3.14`, `0.016`, `1_000.5` | `Float(f64)` | Requires digit after `.` (`1.` is not a float) |
| Boolean | `true`, `false` | `Bool(bool)` | |
| String | `"hello\nworld"` | `String` | Escapes: `\n`, `\r`, `\t`, `\\`, `\"` (unknown `\c` -> `c`) |

### Operators & Punctuation
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Assignment: `=`, `+=`, `-=`, `*=`, `/=`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical: `&&`, `||`, `!` (single `&` and `|` are **lexer errors**)
- Delimiters & misc: `( )`, `{ }`, `[ ]`, `,`, `:`, `::`, `;`, `.`, `->`, `=>`, `?`

---

## 2. Grammar (EBNF)

```ebnf
program        = { item } ;
item           = [ "meow" ] fn_decl
               | struct_decl          (* parsed, ignored by compiler *)
               | enum_decl            (* parsed, ignored by compiler *)
               | impl_block           (* parsed, ignored by compiler *)
               | stmt ;

fn_decl        = "fn" IDENT "(" [ param { "," param } [ "," ] ] ")" [ "->" type ] block ;
param          = [ "mut" ] ( "self" | IDENT ) [ ":" type ] ;
type           = "i32" | "i64" | "f32" | "f64" | "bool" | "string" | "vec2" | "vec3" | IDENT ;

block          = "{" { stmt } [ expr ] "}" ;

stmt           = let_stmt
               | return_stmt
               | break_stmt
               | continue_stmt
               | yield_stmt
               | while_stmt
               | wait_stmt
               | assign_stmt
               | expr [ ";" ] ;

let_stmt       = "let" [ "mut" ] IDENT [ ":" type ] [ "=" expr ] ";" ;
return_stmt    = "return" [ expr ] ";" ;
break_stmt     = "break" ";" ;
continue_stmt  = "continue" ";" ;
yield_stmt     = "yield" ";" ;
while_stmt     = "while" expr block ;
wait_stmt      = "wait" expr ";" ;
assign_stmt    = expr ( "=" | "+=" | "-=" | "*=" | "/=" ) expr ";" ;
```

### Expressions & Precedence (lowest to highest)
1. Logical OR: `||` (short-circuiting)
2. Logical AND: `&&` (short-circuiting)
3. Equality: `==`, `!=`
4. Comparison: `<`, `<=`, `>`, `>=`
5. Additive: `+`, `-`
6. Multiplicative: `*`, `/`, `%`
7. Unary prefix: `!`, `-`
8. Postfix:
   - Function call: `ident(arg1, arg2, ...)`
   - Method call: `expr.method(arg1, ...)`
   - Field access: `expr.x`, `expr.y`, `expr.z`
9. Primary:
   - Literals (`Int`, `Float`, `String`, `true`, `false`)
   - Identifiers (`ident`, `self`)
   - Vector constructors: `vec2(x, y)`, `vec3(x, y, z)`
   - Grouping: `( expr )`
   - Block expression: `{ ... [tail_expr] }`
   - `meow` block expression: `meow { ... [tail_expr] }`
   - `if` expression: `if cond block [ "else" ( if_expr | block ) ]`
   - `match` expression: `match expr "{" { pattern "=>" expr [ "," ] } "}"`

### `match` Patterns
```ebnf
pattern = "_"
        | [ "-" ] ( INT | FLOAT )
        | STRING | "true" | "false"
        | IDENT "::" IDENT [ "(" [ IDENT { "," IDENT } ] ")" ]
        | IDENT ;
```
> **Note:** `Enum::Variant` patterns compile to string equality against `"Variant"` (the variant name only; bindings are ignored). `IDENT` without `::` is a catch-all binding. If no catch-all arm matches, `match` evaluates to `nil`.

---

## 3. Types, Values & Operators

### Runtime `Value` Representation
- `Nil` (falsy; returned by statements, empty blocks, `wait`, void host calls)
- `Bool(bool)`
- `Int(i64)` (checked arithmetic; overflow is a runtime error)
- `Float(f64)`
- `V2(Vec2 { x: f32, y: f32 })`
- `V3(Vec3 { x: f32, y: f32, z: f32 })`
- `String(String)`

> Type annotations (`: i32`, `: f64`, `-> vec2`, etc.) are parsed and **optional**; FustyLang is dynamically typed at runtime.

### Truthiness
- Falsy: `Nil`, `false`, `0`, `0.0`, `NaN`
- Truthy: everything else (including empty string `""` and zero vectors)

### Operator Matrix
- `+`: `Int+Int`, `Float+Float`, `Int+Float`, `Float+Int`, `V2+V2`, `V3+V3`, `String+String` (concatenation)
- `-`: `Int-Int`, `Float-Float`, `Int-Float`, `Float-Int`, `V2-V2`, `V3-V3`
- `*`: numeric (`Int`/`Float`), scalar-vector in either order (`V2 * num`, `num * V2`, `V3 * num`, `num * V3`)
- `/`: numeric (`Int`/`Float`), vector-by-scalar (`V2 / num`, `V3 / num`); division by zero is a runtime error
- `%`: numeric (`Int`/`Float`); modulo by zero is a runtime error
- Unary `-`: `Int`, `Float`, `V2`, `V3`
- `==`, `!=`: compares same types, plus `Int == Float` (via `f64` cast)
- `<`, `<=`, `>`, `>=`: numeric (`Int`/`Float`) and `String` (lexicographic)

### Built-in Vector Fields & Methods
| Member | Applies To | Description |
|--------|------------|-------------|
| `.x`, `.y` | `vec2`, `vec3` | Read or assign (`v.x = 1.0;`, `v.x += 2.0;` on `mut` var) |
| `.z` | `vec3` | Read or assign (`v.z = 1.0;` on `mut` var) |
| `.length()` | `vec2`, `vec3` | Euclidean length (`Float`) |
| `.normalized()` | `vec2`, `vec3` | Unit vector (zero vector if length <= `1e-5`) |
| `.dot(other)` | `vec2`, `vec3` | Dot product (`Float`) |
| `.distance(other)` | `vec2`, `vec3` | `(self - other).length()` (`Float`) |
| `.cross(other)` | `vec3` | Cross product (`Vec3`) |

---

## 4. Scoping, Mutability & Execution Model

### 3-Pass Compilation
1. **Pass 1:** All top-level `fn` declarations are registered (forward calls & recursion work; duplicate names are a compile error; max 255 params).
2. **Pass 2:** Top-level statements are compiled sequentially, followed by `Halt`. Top-level `let` bindings become **globals** (accessible from all functions).
3. **Pass 3:** Function bodies are compiled in isolation with access to globals.

### Mutability
- Enforced at **compile time**.
- `let x = 1;` is immutable; reassigning `x = 2;` or `x += 1;` or `v.x = 1.0;` is a compile error.
- Use `let mut x = 1;` or `fn foo(mut x) { ... }` to allow mutation.
- Field assignment (`v.x = ...` / `v.x += ...`) only works when `v` is a direct variable identifier.

### Coroutines & Step Budget
- `wait <seconds>;` pauses the VM and returns `VmResult::Waiting { duration_sec }`. Execution resumes on the next instruction after the timer expires.
- `yield;` pauses the VM for the current tick and returns `VmResult::Yielded`.
- **Execution budget:** `200_000` bytecode steps per resume without `wait`/`yield` (prevents infinite loops from freezing the engine).
- **Max call depth:** `256` frames.
- **Hot-reload:** Ferrium watches `.fy` file mtime; on valid edit, bytecode is recompiled and VM resets; on syntax/compile error, the previous valid script keeps running.

### `meow` Safety Gate
- `meow { ... }` or `meow fn name(...) { ... }` increments `meow_depth` (enabling `meow_mode = true`) for its dynamic scope.
- Privileged host functions (`raw_teleport`, `raw_teleport3`) fail at runtime unless called inside a `meow` scope.

---

## 5. Ferrium Engine Host API (`ferrium_script`)

All functions below are available as native calls in scripts running inside Ferrium.

### Entity State & Transforms
| Function | Args | Returns | Notes |
|----------|------|---------|-------|
| `get_self_pos()` | 0 | `vec2` | 2D position `(x, z)` |
| `get_self_pos3()` | 0 | `vec3` | 3D position `(x, y, z)` |
| `get_self_rot()` | 0 | `vec3` | Euler rotation |
| `get_self_scale()` | 0 | `vec3` | 3D scale |
| `get_self_vel()` | 0 | `vec2` | 2D velocity |
| `get_self_hp()` | 0 | `Int` | Current HP |
| `get_self_max_hp()` | 0 | `Int` | Max HP |
| `set_self_pos(pos)` | `vec2` | `Nil` | Sets 2D pos (syncs `pos3.x`/`pos3.z`) |
| `set_self_pos3(pos)` | `vec3` | `Nil` | Sets 3D pos (syncs 2D pos) |
| `set_self_rot(rot)` | `vec3` | `Nil` | Sets 3D rotation |
| `set_self_scale(scale)` | `vec3` | `Nil` | Sets 3D scale |
| `set_self_vel(vel)` | `vec2` | `Nil` | Sets 2D velocity |
| `set_self_color(r, g, b[, a])` | 3 or 4 `Int` | `Nil` | RGBA `0..=255` (`a` defaults to `255`) |

### World, Player, Combat & FX
| Function | Args | Returns | Notes |
|----------|------|---------|-------|
| `get_player_pos()` | 0 | `vec2` | Player 2D position |
| `get_player_hp()` | 0 | `Int` | Player HP |
| `spawn_hitbox(offset, size, dmg, dur[, kb])` | `vec2, vec2, Int, num[, num]` | `Nil` | `kb` defaults to `260.0`; `dur >= 0.02` |
| `set_boss_state(state)` | 1 (any) | `Nil` | Sets entity state label string |
| `camera_shake(intensity, duration)` | `num, num` | `Nil` | Triggers camera shake |
| `spawn_particles(pos, count, r, g, b)` | `vec2, Int, Int, Int, Int` | `Nil` | `count` clamped `1..=64`, RGB `0..=255` |
| `log(arg1, ...)` | >= 1 (any) | `Nil` | Space-joined string sent to engine log |

### Input & Time
| Function | Args | Returns | Notes |
|----------|------|---------|-------|
| `dt()` | 0 | `Float` | Frame delta time (seconds) |
| `time()` | 0 | `Float` | Elapsed engine time (seconds) |
| `input_x()` | 0 | `Float` | Horizontal input axis |
| `input_y()` | 0 | `Float` | Vertical input axis |

### Math Builtins
| Function | Args | Returns |
|----------|------|---------|
| `sin(x)`, `cos(x)`, `abs(x)` | 1 `num` | `Float` |
| `sqrt(x)` | 1 `num` | `Float` (clamps `x >= 0.0`) |
| `atan2(y, x)` | 2 `num` | `Float` |
| `min(a, b)`, `max(a, b)` | 2 `num` | `Float` |
| `lerp(a, b, t)` | 3 `num` | `Float` |
| `clamp(x, lo, hi)` | 3 `num` | `Float` |

### Privileged (`meow`) Builtins
| Function | Args | Returns | Notes |
|----------|------|---------|-------|
| `raw_teleport(pos)` | `vec2` | `Nil` | **Requires `meow` mode** |
| `raw_teleport3(pos)` | `vec3` | `Nil` | **Requires `meow` mode** |
| `is_meow()` | 0 | `Bool` | Returns current `meow_mode` flag |

---

## 6. Known Limitations / Gotchas

1. **No `for` loops** — use `while` with a `let mut` counter.
2. **No arrays / lists / indexing** (`[...]` is lexed but not parsed).
3. **No runtime `struct` / `enum` / `impl` instantiation** — declarations are parsed and silently ignored by the compiler. Use top-level `let mut` globals and free `fn`s (or string states with `match`).
4. **No closures or first-class function values** — call targets must be direct function identifiers.
5. **No bitwise operators** — `&` and `|` are lexer errors; only `&&` and `||` exist.
6. **Infinite loops must `wait` or `yield`** — otherwise the VM halts with an execution budget error after `200_000` steps.

---

## 7. Idiomatic Example

```rust
let mut phase = "idle";

fn chase(speed: f64) {
    let dir = (get_player_pos() - get_self_pos()).normalized();
    set_self_vel(dir * speed);
}

while true {
    let dist = get_self_pos().distance(get_player_pos());
    if dist < 80.0 {
        phase = "attack";
        set_boss_state(phase);
        wait 0.2;
        meow {
            raw_teleport(get_player_pos() + vec2(40.0, 0.0));
        }
        spawn_hitbox(vec2(0.0, 0.0), vec2(64.0, 64.0), 25, 0.15);
    } else {
        chase(180.0);
    }
    wait 0.016;
}
```
