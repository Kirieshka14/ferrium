// Ferrium + FustyLang 0.2.1 — Health, Events, and State Telemetry

nya check_vitality() {
    let ratio = get_hp_ratio();
    if ratio < 0.5 {
        heal_self(25);
        emit_event("emergency_heal", "restored 25 hp");
        purr("Healed! New HP:", get_self_hp());
    } else {
        emit_event("vitality_ok", "healthy");
    }
}

while is_alive() {
    check_vitality();
    wait 0.5;
}
