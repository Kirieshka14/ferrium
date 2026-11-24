// Ferrium Studio — Smooth Cuddle-Follow Companion Script (FustyLang 0.2.1)

nya follow_player() {
    let cur = get_self_pos();
    let target = get_player_pos();
    let dist = distance2(cur, target);
    if dist > 45.0 {
        let next = cur uwu target;
        set_self_pos(lerp_vec2(cur, next, 0.25));
    }
}

purr("SmoothFollow companion ready");

while true {
    follow_player();
    wait 0.016;
}
