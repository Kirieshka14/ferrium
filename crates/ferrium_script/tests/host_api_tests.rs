use ferrium_ecs::Entity;
use ferrium_math::{Color, Vec2, Vec3};
use ferrium_script::{
    FerriumHost, ScriptCommand, ScriptComponent, ScriptSnapshot, VmResult,
};

fn sample_snapshot() -> ScriptSnapshot {
    ScriptSnapshot {
        self_entity: Entity { id: 42, generation: 1 },
        self_pos: Vec2::new(10.0, 20.0),
        self_pos3: Vec3::new(10.0, 5.0, 20.0),
        self_rot: Vec3::new(0.1, 0.2, 0.3),
        self_scale: Vec3::new(1.0, 2.0, 3.0),
        self_vel: Vec2::new(3.0, -4.0),
        self_hp: 60,
        self_max_hp: 100,
        player_pos: Vec2::new(13.0, 24.0),
        player_hp: 85,
        input_x: 0.75,
        input_y: -0.25,
        dt: 1.0 / 60.0,
        elapsed_time: 12.5,
        frame_count: 750,
        jit_enabled: true,
        jit_compiled_count: 4,
    }
}

fn run_expr(src: &str) -> (VmResult, FerriumHost) {
    let mut comp = ScriptComponent::from_source("host_test.fy", src).expect("compile failed");
    let mut host = FerriumHost::new(sample_snapshot());
    let res = comp.step(&mut host).expect("runtime failed");
    (res, host)
}

#[test]
fn test_entity_and_health_queries() {
    let (res, _) = run_expr(
        "return get_entity_id() + get_self_hp() + get_self_max_hp() + get_player_hp();",
    );
    match res {
        VmResult::Done(v) => assert_eq!(v.to_string(), "287"),
        other => panic!("unexpected {:?}", other),
    }
}

#[test]
fn test_hp_ratio_and_is_alive() {
    let (res, _) = run_expr("if is_alive() { return get_hp_ratio(); } return -1.0;");
    match res {
        VmResult::Done(v) => assert!((v.as_f64().unwrap() - 0.6).abs() < 1e-6),
        other => panic!("unexpected {:?}", other),
    }
}

#[test]
fn test_distance_and_dir_to_player() {
    let (res, _) = run_expr("return distance_to_player();");
    match res {
        VmResult::Done(v) => assert!((v.as_f64().unwrap() - 5.0).abs() < 1e-5),
        other => panic!("unexpected {:?}", other),
    }
    let (res2, _) = run_expr("return dir_to_player();");
    match res2 {
        VmResult::Done(v) => assert_eq!(v.to_string(), "vec2(0.60, 0.80)"),
        other => panic!("unexpected {:?}", other),
    }
}

#[test]
fn test_transform_3d_getters() {
    let (p3, _) = run_expr("return get_self_pos3();");
    let (r3, _) = run_expr("return get_self_rot();");
    let (s3, _) = run_expr("return get_self_scale();");
    assert!(matches!(p3, VmResult::Done(v) if v.to_string() == "vec3(10.00, 5.00, 20.00)"));
    assert!(matches!(r3, VmResult::Done(v) if v.to_string() == "vec3(0.10, 0.20, 0.30)"));
    assert!(matches!(s3, VmResult::Done(v) if v.to_string() == "vec3(1.00, 2.00, 3.00)"));
}

#[test]
fn test_velocity_get_and_set() {
    let (_, host) = run_expr("let v = get_self_vel(); set_self_vel(v * 2.0);");
    assert!(host.commands.iter().any(|c| matches!(
        c,
        ScriptCommand::SetVel(e, v) if e.id == 42 && (v.x - 6.0).abs() < 1e-4 && (v.y + 8.0).abs() < 1e-4
    )));
}

#[test]
fn test_set_and_translate_self_2d() {
    let (res, host) = run_expr("set_self_pos(vec2(1.0, 2.0)); return translate_self(vec2(3.0, 4.0));");
    assert!(matches!(res, VmResult::Done(v) if v.to_string() == "vec2(4.00, 6.00)"));
    assert_eq!(host.snapshot.self_pos, Vec2::new(4.0, 6.0));
    assert_eq!(host.commands.len(), 2);
}

#[test]
fn test_set_and_translate_self_3d() {
    let (res, host) = run_expr("set_self_pos3(vec3(1.0, 2.0, 3.0)); return translate_self3(vec3(4.0, 5.0, 6.0));");
    assert!(matches!(res, VmResult::Done(v) if v.to_string() == "vec3(5.00, 7.00, 9.00)"));
    assert!(host.commands.iter().any(|c| matches!(
        c,
        ScriptCommand::SetPos3(_, p) if *p == Vec3::new(5.0, 7.0, 9.0)
    )));
}

#[test]
fn test_set_and_rotate_self() {
    let (res, host) = run_expr("set_self_rot(vec3(0.0, 1.0, 0.0)); return rotate_self(vec3(0.5, 0.5, 0.0));");
    assert!(matches!(res, VmResult::Done(v) if v.to_string() == "vec3(0.50, 1.50, 0.00)"));
    assert!(host.commands.iter().any(|c| matches!(
        c,
        ScriptCommand::SetRot(_, r) if (r.x - 0.5).abs() < 1e-4 && (r.y - 1.5).abs() < 1e-4
    )));
}

#[test]
fn test_set_and_scale_self() {
    let (res, host) = run_expr("set_self_scale(vec3(2.0, 3.0, 4.0)); return scale_self(0.5);");
    assert!(matches!(res, VmResult::Done(v) if v.to_string() == "vec3(1.00, 1.50, 2.00)"));
    assert!(host.commands.iter().any(|c| matches!(
        c,
        ScriptCommand::SetScale(_, s) if *s == Vec3::new(1.0, 1.5, 2.0)
    )));
}

#[test]
fn test_set_self_color_rgb_and_rgba() {
    let (_, host) = run_expr("set_self_color(300, -10, 128); set_self_color(10, 20, 30, 120);");
    assert!(matches!(host.commands[0], ScriptCommand::SetColor(_, c) if c == Color::rgba(255, 0, 128, 255)));
    assert!(matches!(host.commands[1], ScriptCommand::SetColor(_, c) if c == Color::rgba(10, 20, 30, 120)));
}

#[test]
fn test_heal_and_damage_self() {
    let (res, host) = run_expr("let h1 = heal_self(50); let h2 = damage_self(120); return h1 * 1000 + h2;");
    assert!(matches!(res, VmResult::Done(v) if v.to_string() == "100000"));
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::Heal { amount: 50, .. })));
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::Damage { amount: 120, .. })));
}

#[test]
fn test_emit_event_variants() {
    let (_, host) = run_expr("emit_event("boss_phase"); emit_event("loot", "crystal");");
    assert!(matches!(&host.commands[0], ScriptCommand::EmitEvent { name, payload, .. } if name == "boss_phase" && payload.is_empty()));
    assert!(matches!(&host.commands[1], ScriptCommand::EmitEvent { name, payload, .. } if name == "loot" && payload == "crystal"));
}

#[test]
fn test_camera_shake_and_particles() {
    let (_, host) = run_expr("camera_shake(8.5, 0.4); spawn_particles(vec2(5.0, 6.0), 100, 255, 128, 64);");
    assert!(matches!(host.commands[0], ScriptCommand::CameraShake { intensity, duration } if (intensity - 8.5).abs() < 1e-4 && (duration - 0.4).abs() < 1e-4));
    assert!(matches!(host.commands[1], ScriptCommand::SpawnParticles { count: 64, color, .. } if color == Color::rgb(255, 128, 64)));
}

#[test]
fn test_spawn_hitbox_knockback() {
    let (_, host) = run_expr("spawn_hitbox(vec2(1.0, 0.0), vec2(20.0, 20.0), 15, 0.1); spawn_hitbox(vec2(2.0, 0.0), vec2(10.0, 10.0), 30, 0.001, 500.0);");
    assert!(matches!(host.commands[0], ScriptCommand::SpawnHitbox { damage: 15, knockback, .. } if (knockback - 260.0).abs() < 1e-3));
    assert!(matches!(host.commands[1], ScriptCommand::SpawnHitbox { damage: 30, duration, knockback, .. } if (duration - 0.02).abs() < 1e-4 && (knockback - 500.0).abs() < 1e-3));
}

#[test]
fn test_time_input_and_frame_intrinsics() {
    let (res, _) = run_expr("return time() + input_x() + input_y() + get_frame_count();");
    match res {
        VmResult::Done(v) => assert!((v.as_f64().unwrap() - 763.0).abs() < 1e-6),
        other => panic!("unexpected {:?}", other),
    }
    let (dt_res, _) = run_expr("return dt();");
    match dt_res {
        VmResult::Done(v) => assert!((v.as_f64().unwrap() - 1.0 / 60.0).abs() < 1e-6),
        other => panic!("unexpected {:?}", other),
    }
}

#[test]
fn test_jit_introspection_intrinsics() {
    let (res, _) = run_expr("if jit_enabled() { return jit_compiled_count(); } return -1;");
    assert!(matches!(res, VmResult::Done(v) if v.to_string() == "4"));
}
