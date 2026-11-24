// Ferrium + FustyLang 0.2.1 — Native x86-64 JIT Hot-Loop Benchmark

nya fib_iter(n: i64) -> i64 {
    let mut a = 0;
    let mut b = 1;
    let mut i = 0;
    while i < n {
        let next = a + b;
        a = b;
        b = next;
        i += 1;
    }
    return a;
}

let mut round = 0;
let mut last = 0;
while round < 12 {
    last = fib_iter(40);
    round += 1;
}
purr("fib_iter(40) =", last, "| JIT compiled =", jit_compiled_count());
