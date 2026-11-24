// Ferrium + FustyLang 0.2.1 — 3D Orbital Motion Script

let radius = 3.2;
let angular_speed = 1.4;

nya compute_orbit(t: f64) {
    let angle = t * angular_speed;
    let x = cos(angle) * radius;
    let z = sin(angle) * radius;
    let y = 1.0 + sin(angle * 2.0) * 0.4;
    set_self_pos3(vec3(x, y, z));
    set_self_rot(vec3(0.0, -angle, 0.0));
}

purr("OrbitSwarm active for entity", get_entity_id());

while true {
    compute_orbit(time());
    wait 0.016;
}
