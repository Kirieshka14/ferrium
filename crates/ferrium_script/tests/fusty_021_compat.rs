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
