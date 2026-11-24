// Ferrium Studio — Sinusoidal RGB Pulse Script (FustyLang 0.2.1)

nya pulse_color() {
    let t = time() * 2.5;
    let r = floor(128.0 + sin(t) * 120.0);
    let g = floor(128.0 + sin(t + 2.09) * 120.0);
    let b = floor(128.0 + sin(t + 4.18) * 120.0);
    set_self_color(r, g, b);
}

while true {
    pulse_color();
    wait 0.033;
}
