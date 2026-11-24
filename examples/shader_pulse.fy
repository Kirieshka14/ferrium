// Procedural RGB pulse using smoothstep, sin, and set_self_color
let t = time();
let wave = 0.5 + 0.5 * sin(t * 3.0);
let glow = smoothstep(0.0, 1.0, wave);
let r = clamp(glow * 255.0, 40.0, 255.0);
let g = clamp((1.0 - glow) * 200.0, 30.0, 220.0);
set_self_color(r, g, 240);
purr("Pulse glow", glow);
return glow;
