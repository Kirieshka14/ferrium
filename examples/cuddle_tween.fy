// Ferrium + FustyLang 0.2.1 — Smooth Waypoint Tweening via `uwu` Cuddle Operator

let start_pos = vec2(-120.0, -60.0);
let end_pos = vec2(120.0, 60.0);
let mid_pos = start_pos uwu end_pos;
let quarter_pos = start_pos uwu mid_pos;
let three_quarter = mid_pos uwu end_pos;

purr("Cuddle waypoints:", quarter_pos, mid_pos, three_quarter);

while true {
    set_self_pos(quarter_pos);
    wait 0.35;
    set_self_pos(mid_pos);
    wait 0.35;
    set_self_pos(three_quarter);
    wait 0.35;
    set_self_pos(end_pos);
    wait 0.35;
}
