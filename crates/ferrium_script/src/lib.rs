pub mod compiler;
pub mod component;
pub mod error;
pub mod host;
pub mod jit;

pub use compiler::{compile_fusty_detailed, compile_fusty_source, CompileArtifact, FunctionSummary};
pub use component::ScriptComponent;
pub use error::{is_baka_message, CompileStage, ScriptError};
pub use host::{FerriumHost, ScriptCommand, ScriptSnapshot};
pub use jit::{
    check_jit_eligibility, is_supported_platform, JitFallbackDiagnostic, JitMode, JitPolicy,
    JitStats, JitTelemetry, JitValueKind,
};
pub use fusty_vm::gc::{GcConfig, GcStats};
pub use fusty_vm::vm::VmResult;

#[cfg(test)]
mod tests {
    use super::*;
    use ferrium_ecs::Entity;
    use ferrium_math::{Vec2, Vec3};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_snapshot() -> ScriptSnapshot {
        ScriptSnapshot {
            self_entity: Entity {
                id: 1,
                generation: 0,
            },
            self_pos: Vec2::new(200.0, 0.0),
            self_pos3: Vec3::new(200.0, 0.0, 0.0),
            self_rot: Vec3::ZERO,
            self_scale: Vec3::ONE,
            self_vel: Vec2::ZERO,
            self_hp: 1000,
            self_max_hp: 1000,
            player_pos: Vec2::new(-100.0, 0.0),
            player_hp: 100,
            input_x: 0.0,
            input_y: 0.0,
            dt: 1.0 / 60.0,
            elapsed_time: 1.5,
            frame_count: 42,
            jit_enabled: true,
            jit_compiled_count: 0,
        }
    }

    #[test]
    fn test_script_coroutine_and_commands() {
        let src = r#"
            set_boss_state("windup");
            wait 0.25;
            let dir = (get_player_pos() - get_self_pos()).normalized();
            set_self_vel(dir * 300.0);
            spawn_hitbox(dir * 35.0, vec2(60.0, 60.0), 20, 0.2);
            log("strike", 1);
            purr("strike complete");
        "#;
        let mut comp = ScriptComponent::from_source("inline.fy", src).unwrap();
        let mut host = FerriumHost::new(test_snapshot());

        match comp.step(&mut host).unwrap() {
            VmResult::Waiting { duration_sec } => assert!((duration_sec - 0.25).abs() < 1e-6),
            other => panic!("Expected wait, got {:?}", other),
        }
        assert!(matches!(
            host.commands.as_slice(),
            [ScriptCommand::SetState { state, .. }] if state == "windup"
        ));

        let mut host2 = FerriumHost::new(test_snapshot());
        assert!(matches!(comp.step(&mut host2).unwrap(), VmResult::Done(_)));
        assert!(host2
            .commands
            .iter()
            .any(|c| matches!(c, ScriptCommand::SpawnHitbox { damage: 20, .. })));
        assert!(host2
            .commands
            .iter()
            .any(|c| matches!(c, ScriptCommand::Log(m) if m == "strike 1")));
        assert!(host2
            .commands
            .iter()
            .any(|c| matches!(c, ScriptCommand::Purr(m) if m == "strike complete")));
        assert_eq!(comp.purr_count, 1);
    }

    #[test]
    fn test_meow_teleport_gate() {
        let mut comp_safe =
            ScriptComponent::from_source("bad.fy", "raw_teleport(vec2(10.0, 20.0));").unwrap();
        let mut host = FerriumHost::new(test_snapshot());
        assert!(comp_safe.step(&mut host).is_err());

        let mut comp_meow = ScriptComponent::from_source(
            "ok.fy",
            "meow { raw_teleport(vec2(10.0, 20.0)); }",
        )
        .unwrap();
        let mut host2 = FerriumHost::new(test_snapshot());
        comp_meow.step(&mut host2).unwrap();
        assert!(host2.used_meow);
        assert!(host2
            .commands
            .iter()
            .any(|c| matches!(c, ScriptCommand::RawTeleport(_, p) if *p == Vec2::new(10.0, 20.0))));
    }

    #[test]
    fn test_hot_reload_resilience_on_syntax_error() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_path = std::env::temp_dir().join(format!("ferrium_hot_reload_{}.fy", nanos));
        fs::write(&tmp_path, "set_boss_state(\"idle\");").unwrap();

        let mut comp = ScriptComponent::from_file(&tmp_path).unwrap();
        let orig_len = comp.chunk.code.len();

        // Broken update must fail reload and preserve working chunk
        fs::write(&tmp_path, "let broken = ;;;").unwrap();
        let err_res = comp.try_hot_reload(true).unwrap();
        assert!(err_res.is_err());
        assert!(comp.last_error.is_some());
        assert_eq!(comp.chunk.code.len(), orig_len);

        // Valid update succeeds and bumps reload_count
        fs::write(&tmp_path, "set_boss_state(\"enrage\"); wait 0.5;").unwrap();
        let ok_res = comp.try_hot_reload(true).unwrap();
        assert_eq!(ok_res.unwrap(), 1);
        assert!(comp.last_error.is_none());

        let _ = fs::remove_file(&tmp_path);
    }
}
