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

#[test]
fn test_uwu_operator_floats() {
    let mut comp = ScriptComponent::from_source("uwu_flt.fy", "return 2.5 uwu 7.5;").unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    match comp.step(&mut host).unwrap() {
        VmResult::Done(v) => assert!((v.as_f64().unwrap() - 5.0).abs() < 1e-6),
        other => panic!("Unexpected {:?}", other),
    }
}

#[test]
fn test_uwu_operator_vec2() {
    let mut comp = ScriptComponent::from_source(
        "uwu_v2.fy",
        "return vec2(10.0, 20.0) uwu vec2(30.0, 60.0);",
    ).unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    match comp.step(&mut host).unwrap() {
        VmResult::Done(v) => assert_eq!(v.to_string(), "vec2(20.00, 40.00)"),
        other => panic!("Unexpected {:?}", other),
    }
}

#[test]
fn test_uwu_operator_vec3() {
    let mut comp = ScriptComponent::from_source(
        "uwu_v3.fy",
        "return vec3(2.0, 4.0, 6.0) uwu vec3(8.0, 12.0, 14.0);",
    ).unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    match comp.step(&mut host).unwrap() {
        VmResult::Done(v) => assert_eq!(v.to_string(), "vec3(5.00, 8.00, 10.00)"),
        other => panic!("Unexpected {:?}", other),
    }
}

#[test]
fn test_uwu_precedence() {
    let mut comp = ScriptComponent::from_source(
        "uwu_prec.fy",
        "return 1 + 3 uwu 5 + 7 == 8.0;",
    ).unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    match comp.step(&mut host).unwrap() {
        VmResult::Done(v) => assert_eq!(v.to_string(), "true"),
        other => panic!("Unexpected {:?}", other),
    }
}

#[test]
fn test_uwu_type_mismatch_error() {
    let mut comp = ScriptComponent::from_source("uwu_err.fy", "return 10 uwu vec2(1.0, 2.0);").unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    let err = comp.step(&mut host).unwrap_err();
    assert!(matches!(err, ScriptError::Runtime { .. }));
}

#[test]
fn test_baka_trap_caught_safely() {
    let mut comp = ScriptComponent::from_source("baka.fy", "baka!(\"invalid state\");").unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    let err = comp.step(&mut host).unwrap_err();
    assert!(err.is_baka(), "Expected baka error, got {:?}", err);
    assert_eq!(comp.baka_count, 1);
    assert_eq!(comp.status, "baka!");
    assert!(comp.last_baka.is_some());
}

#[test]
fn test_recover_from_baka_preserves_jit_policy() {
    let mut comp = ScriptComponent::from_source_with_jit(
        "baka_rec.fy",
        "baka!(\"oops\");",
        JitPolicy::Eager,
    ).unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    assert!(comp.step(&mut host).is_err());
    comp.recover_from_baka();
    assert_eq!(comp.status, "recovered");
    assert!(comp.last_error.is_none());
    assert!(comp.last_baka.is_none());
    assert_eq!(comp.jit_policy, JitPolicy::Eager);
}

#[test]
fn test_is_baka_message_helper() {
    assert!(is_baka_message("[BAKA! >///<] bad value"));
    assert!(!is_baka_message("Unknown native host function 'foo'"));
}

#[test]
fn test_purr_single_arg() {
    let mut comp = ScriptComponent::from_source("purr1.fy", "purr(\"hello ferrium\");").unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    comp.step(&mut host).unwrap();
    assert_eq!(comp.purr_count, 1);
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::Purr(s) if s == "hello ferrium")));
}

#[test]
fn test_purr_multi_arg() {
    let mut comp = ScriptComponent::from_source("purr2.fy", "purr(\"hp =\", 80, true);").unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    comp.step(&mut host).unwrap();
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::Purr(s) if s == "hp = 80 true")));
}

#[test]
fn test_purr_zero_args_error() {
    let mut comp = ScriptComponent::from_source("purr0.fy", "purr();").unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    assert!(comp.step(&mut host).is_err());
}

#[test]
fn test_nested_block_comments() {
    let src = "/* outer /* inner */ still outer */\nreturn 7;";
    let mut comp = ScriptComponent::from_source("nested.fy", src).unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    match comp.step(&mut host).unwrap() {
        VmResult::Done(v) => assert_eq!(v.to_string(), "7"),
        other => panic!("Unexpected {:?}", other),
    }
}

#[test]
fn test_coroutine_yield_and_resume() {
    let src = "purr(\"step 1\");\nyield;\npurr(\"step 2\");";
    let mut comp = ScriptComponent::from_source("yield.fy", src).unwrap();
    let mut host1 = FerriumHost::new(base_snapshot());
    assert_eq!(comp.step(&mut host1).unwrap(), VmResult::Yielded);
    assert_eq!(comp.status, "yielded");

    let mut host2 = FerriumHost::new(base_snapshot());
    assert!(matches!(comp.step(&mut host2).unwrap(), VmResult::Done(_)));
    assert_eq!(comp.purr_count, 2);
}

#[test]
fn test_coroutine_wait_sets_timer() {
    let mut comp = ScriptComponent::from_source("wait.fy", "wait 0.42;").unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    match comp.step(&mut host).unwrap() {
        VmResult::Waiting { duration_sec } => assert!((duration_sec - 0.42).abs() < 1e-6),
        other => panic!("Unexpected {:?}", other),
    }
    assert!((comp.wait_remaining - 0.42).abs() < 1e-6);
}

#[test]
fn test_infinite_loop_hits_step_budget() {
    let mut comp = ScriptComponent::from_source("inf.fy", "while true { let x = 1; }").unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    let err = comp.step(&mut host).unwrap_err();
    assert!(matches!(err, ScriptError::Runtime { is_baka: false, .. }));
    assert_eq!(comp.status, "error");
}

#[test]
fn test_immutability_compile_error() {
    let err = compile_fusty_detailed("let x = 5; x = 10;").unwrap_err();
    assert!(matches!(err, ScriptError::Compile { stage: CompileStage::Bytecode, .. }));
}

#[test]
fn test_lexer_error_line_and_col() {
    let err = compile_fusty_detailed("let s = \"unterminated").unwrap_err();
    assert!(matches!(err, ScriptError::Compile { stage: CompileStage::Lexer, line: 1, .. }));
}

#[test]
fn test_repo_player_script_compiles() {
    let src = std::fs::read_to_string("../../scripts/Player.fy").unwrap();
    let art = compile_fusty_detailed(&src).unwrap();
    assert_eq!(art.functions.len(), 2);
}

#[test]
fn test_repo_game_manager_script_compiles() {
    let src = std::fs::read_to_string("../../scripts/GameManager.fy").unwrap();
    assert!(compile_fusty_source(&src).is_ok());
}

#[test]
fn test_repo_spin_model_script_compiles() {
    let src = std::fs::read_to_string("../../scripts/SpinModel.fy").unwrap();
    let art = compile_fusty_detailed(&src).unwrap();
    assert_eq!(art.functions.len(), 1);
}

#[test]
fn test_repo_patrol_agent_script_compiles() {
    let src = std::fs::read_to_string("../../scripts/patrol_agent.fy").unwrap();
    let art = compile_fusty_detailed(&src).unwrap();
    assert_eq!(art.functions.len(), 1);
}

#[test]
fn test_repo_boss_arena_example_compiles() {
    let src = std::fs::read_to_string("../../examples/boss_arena.fy").unwrap();
    let art = compile_fusty_detailed(&src).unwrap();
    assert_eq!(art.functions.len(), 1);
}

#[test]
fn test_repo_boss_fight_example_compiles() {
    let src = std::fs::read_to_string("../../examples/boss_fight.fy").unwrap();
    assert!(compile_fusty_source(&src).is_ok());
}

#[test]
fn test_repo_jit_physics_example_compiles() {
    let src = std::fs::read_to_string("../../examples/jit_physics.fy").unwrap();
    let art = compile_fusty_detailed(&src).unwrap();
    assert_eq!(art.functions.len(), 2);
    assert!(art.jit_eligible_count() >= 1);
}

#[test]
fn test_repo_cuddle_tween_example_runs() {
    let src = std::fs::read_to_string("../../examples/cuddle_tween.fy").unwrap();
    let mut comp = ScriptComponent::from_source("cuddle_tween.fy", &src).unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    assert!(matches!(comp.step(&mut host).unwrap(), VmResult::Waiting { .. }));
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::Purr(_))));
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::SetPos(_, _))));
}

#[test]
fn test_repo_baka_guard_example_valid_and_trap() {
    let src = std::fs::read_to_string("../../examples/baka_guard.fy").unwrap();
    let mut ok_comp = ScriptComponent::from_source("baka_guard.fy", &src).unwrap();
    let mut ok_host = FerriumHost::new(base_snapshot());
    assert!(matches!(ok_comp.step(&mut ok_host).unwrap(), VmResult::Done(_)));

    let mut bad_snap = base_snapshot();
    bad_snap.self_hp = -10;
    let mut bad_comp = ScriptComponent::from_source("baka_guard.fy", &src).unwrap();
    let mut bad_host = FerriumHost::new(bad_snap);
    let err = bad_comp.step(&mut bad_host).unwrap_err();
    assert!(err.is_baka());
}

#[test]
fn test_repo_orbit_swarm_example_runs() {
    let src = std::fs::read_to_string("../../examples/orbit_swarm.fy").unwrap();
    let mut comp = ScriptComponent::from_source("orbit_swarm.fy", &src).unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    assert!(matches!(comp.step(&mut host).unwrap(), VmResult::Waiting { .. }));
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::SetPos3(_, _))));
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::SetRot(_, _))));
}

#[test]
fn test_repo_event_bus_example_heals_low_hp() {
    let src = std::fs::read_to_string("../../examples/event_bus.fy").unwrap();
    let mut snap = base_snapshot();
    snap.self_hp = 30;
    let mut comp = ScriptComponent::from_source("event_bus.fy", &src).unwrap();
    let mut host = FerriumHost::new(snap);
    assert!(matches!(comp.step(&mut host).unwrap(), VmResult::Waiting { .. }));
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::Heal { amount: 25, .. })));
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::EmitEvent { name, .. } if name == "emergency_heal")));
}

#[test]
fn test_repo_jit_benchmark_example_runs() {
    let src = std::fs::read_to_string("../../examples/jit_benchmark.fy").unwrap();
    let mut comp = ScriptComponent::from_source_with_jit("jit_benchmark.fy", &src, JitPolicy::Eager).unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    assert!(matches!(comp.step(&mut host).unwrap(), VmResult::Done(_)));
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::Purr(s) if s.contains("102334155"))));
}

#[test]
fn test_repo_orbit_camera_script_runs() {
    let src = std::fs::read_to_string("../../scripts/OrbitCamera.fy").unwrap();
    let mut comp = ScriptComponent::from_source("OrbitCamera.fy", &src).unwrap();
    let mut host = FerriumHost::new(base_snapshot());
    assert!(matches!(comp.step(&mut host).unwrap(), VmResult::Waiting { .. }));
    assert!(host.commands.iter().any(|c| matches!(c, ScriptCommand::SetPos3(_, _))));
}
