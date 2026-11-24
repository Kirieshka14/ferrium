// Wave spawner showcase using FustyLang 0.2.1 `nya`, `purr`, and `emit_event`
nya wave_enemy_count(wave: i64) -> i64 {
    return 3 + wave * 2;
}

let wave = 3;
let count = wave_enemy_count(wave);
emit_event("spawn_wave", "tier_3");
purr("Wave", wave, "spawning", count, "units");
return count;
