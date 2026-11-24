// Ferrium + FustyLang 0.2.1 — Defensive Invariant Guard with `baka!(...)`
// Ferrium catches `baka!` traps safely without crashing the engine or editor.

nya validate_hp(hp: i64) -> i64 {
    if hp < 0 {
        baka!("HP cannot be negative!");
    }
    return hp;
}

let hp = validate_hp(get_self_hp());
purr("Validated entity HP:", hp, "ratio:", get_hp_ratio());
emit_event("hp_verified", "ok");
