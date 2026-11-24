// Ferrium + FustyLang 0.2.1 — Native x86-64 JIT Scalar Physics Kernel
// Hot pure-scalar `nya` functions compile natively via fusty_jit!

nya integrate_damped(v: f64, accel: f64, drag: f64, steps: i64) -> f64 {
    let mut vel = v;
    let mut i = 0;
    while i < steps {
        vel = (vel + accel) * drag;
        i += 1;
    }
    return vel;
}

nya energy_sum(n: i64) -> i64 {
    let mut acc = 0;
    let mut i = 1;
    while i <= n {
        acc = acc + i * i;
        i += 1;
    }
    return acc;
}

purr("JIT physics kernel online, jit_enabled =", jit_enabled());

let mut tick = 0;
while tick < 10 {
    let v = integrate_damped(12.5, 0.4, 0.96, 64);
    let e = energy_sum(50);
    if tick == 9 {
        purr("JIT kernel result: v =", v, "energy =", e, "compiled =", jit_compiled_count());
    }
    tick += 1;
    wait 0.016;
}
