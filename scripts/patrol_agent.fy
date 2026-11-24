// ============================================================================
// Ferrium Studio — Starter Patrol Agent Script (FustyLang 0.2.1 .fy)
// Edit & save (Ctrl+S) in the Fusty IDE tab to hot-reload live!
// ============================================================================

nya move_toward(target: vec2, speed: f64) {
    let pos = get_self_pos();
    let delta = target - pos;
    if delta.length() > 12.0 {
        set_self_vel(delta.normalized() * speed);
    } else {
        set_self_vel(vec2(0.0, 0.0));
    }
}

purr("PatrolAgent online (FustyLang 0.2.1)");

let wp_a = vec2(140.0, -90.0);
let wp_b = vec2(140.0, 90.0);
let wp_c = vec2(260.0, 0.0);
// Compute midpoint between wp_a and wp_c using FustyLang 0.2.1 uwu operator
let wp_mid = wp_a uwu wp_c;

while true {
    set_boss_state("patrol_a");
    move_toward(wp_a, 130.0);
    wait 0.65;

    set_boss_state("patrol_b");
    move_toward(wp_b, 130.0);
    spawn_particles(get_self_pos(), 6, 56, 225, 245);
    wait 0.65;

    set_boss_state("patrol_mid");
    move_toward(wp_mid, 140.0);
    wait 0.45;

    set_boss_state("patrol_c");
    move_toward(wp_c, 150.0);
    wait 0.60;

    // Try uncommenting this meow block and saving (Ctrl+S) during Play mode:
    // meow {
    //     raw_teleport(wp_a uwu wp_b);
    //     spawn_particles(get_self_pos(), 14, 235, 75, 225);
    // }
}
