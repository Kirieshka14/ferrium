use ferrium_ecs::Entity;
use ferrium_math::{Vec2, Vec3};
use ferrium_script::{
    compile_fusty_detailed, compile_fusty_source, is_baka_message, CompileStage, FerriumHost,
    JitPolicy, ScriptCommand, ScriptComponent, ScriptError, ScriptSnapshot, VmResult,
};

fn base_snapshot() -> ScriptSnapshot {
    ScriptSnapshot {
        self_entity: Entity { id: 7, generation: 0 },
        self_pos: Vec2::new(100.0, 50.0),
        self_pos3: Vec3::new(100.0, 2.0, 50.0),
        self_rot: Vec3::ZERO,
        self_scale: Vec3::ONE,
        self_vel: Vec2::ZERO,
        self_hp: 80,
        self_max_hp: 100,
        player_pos: Vec2::new(0.0, 0.0),
        player_hp: 100,
        input_x: 1.0,
        input_y: -0.5,
        dt: 1.0 / 60.0,
        elapsed_time: 2.5,
        frame_count: 150,
        jit_enabled: true,
        jit_compiled_count: 0,
    }
}

#[test]
fn test_nya_function_declaration_and_return() {
    let mut comp = ScriptComponent::from_source(
        "nya_basic.fy",
        "nya double(x: i64) -> i64 { return x * 2; }\nreturn double(21);",
    ).unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    match comp.step(&mut host).unwrap() {
        VmResult::Done(v) => assert_eq!(v.to_string(), "42"),
        other => panic!("Unexpected {:?}", other),
    }
}

#[test]
fn test_meow_nya_function_enables_meow_mode() {
    let src = "meow nya blink() { raw_teleport(vec2(12.0, 34.0)); }\nblink();";
    let mut comp = ScriptComponent::from_source("meow_nya.fy", src).unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    comp.step(&mut host).unwrap();
    assert!(host.used_meow);
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::RawTeleport(_, p) if *p == Vec2::new(12.0, 34.0))));
}

#[test]
fn test_legacy_fn_keyword_rejected() {
    let err = compile_fusty_detailed("fn old_style() { return 1; }").unwrap_err();
    assert!(matches!(err, ScriptError::Compile { stage: CompileStage::Parser, .. }));
}

#[test]
fn test_uwu_operator_integers_returns_float_midpoint() {
    let mut comp = ScriptComponent::from_source("uwu_int.fy", "return 10 uwu 21;").unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    match comp.step(&mut host).unwrap() {
        VmResult::Done(v) => assert!((v.as_f64().unwrap() - 15.5).abs() < 1e-6),
        other => panic!("Unexpected {:?}", other),
    }
}
