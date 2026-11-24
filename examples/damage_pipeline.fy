// Safe combat damage pipeline with JIT-eligible armor formula and baka! invariant guard
nya mitigate(raw_dmg: i64, armor: i64) -> i64 {
    let reduced = raw_dmg - armor;
    if reduced < 1 {
        return 1;
    }
    return reduced;
}

let dmg = mitigate(35, 10);
if dmg <= 0 {
    baka!("mitigated damage must stay positive");
}
let remaining = damage_self(dmg);
camera_shake(4.0, 0.15);
purr("Applied damage:", dmg, "remaining HP:", remaining);
return remaining;
