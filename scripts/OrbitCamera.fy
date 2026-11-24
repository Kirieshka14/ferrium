// Ferrium Studio — Smooth Orbital Motion Helper (FustyLang 0.2.1)

let orbit_radius = 4.5;

nya step_orbit() {
    let t = time();
    let p = vec3(cos(t * 0.6) * orbit_radius, 1.5, sin(t * 0.6) * orbit_radius);
    set_self_pos3(p);
}

while true {
    step_orbit();
    wait 0.016;
}
