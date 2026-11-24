// Ferrium Studio — Passive Health Regeneration (FustyLang 0.2.1)

let regen_amount = 5;

nya tick_regen() {
    if is_alive() {
        if get_hp_ratio() < 1.0 {
            heal_self(regen_amount);
            purr("Regenerated HP ->", get_self_hp());
        }
    }
}

while true {
    tick_regen();
    wait 1.0;
}
