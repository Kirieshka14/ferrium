use ferrium_script::{
    check_jit_eligibility, compile_fusty_detailed, is_supported_platform, FerriumHost, JitMode,
    JitPolicy, JitStats, JitTelemetry, ScriptComponent, ScriptSnapshot, VmResult,
};

#[test]
fn test_jit_policy_default_and_mode() {
    assert_eq!(JitPolicy::default(), JitPolicy::Auto);
    assert_eq!(
        JitPolicy::Auto.to_jit_mode(),
        JitMode::Auto { hot_threshold: JitPolicy::DEFAULT_HOT_THRESHOLD }
    );
    assert_eq!(JitPolicy::Eager.to_jit_mode(), JitMode::Eager);
    assert_eq!(JitPolicy::Disabled.to_jit_mode(), JitMode::Disabled);
}
