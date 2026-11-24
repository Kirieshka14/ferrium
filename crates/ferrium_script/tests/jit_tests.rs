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

#[test]
fn test_jit_policy_cycle() {
    assert_eq!(JitPolicy::Auto.cycle(), JitPolicy::Eager);
    assert_eq!(JitPolicy::Eager.cycle(), JitPolicy::Disabled);
    assert_eq!(JitPolicy::Disabled.cycle(), JitPolicy::Auto);
}

#[test]
fn test_jit_policy_parse() {
    assert_eq!(JitPolicy::parse("auto"), Some(JitPolicy::Auto));
    assert_eq!(JitPolicy::parse("EAGER"), Some(JitPolicy::Eager));
    assert_eq!(JitPolicy::parse("on"), Some(JitPolicy::Eager));
    assert_eq!(JitPolicy::parse("always"), Some(JitPolicy::Eager));
    assert_eq!(JitPolicy::parse("off"), Some(JitPolicy::Disabled));
    assert_eq!(JitPolicy::parse("disabled"), Some(JitPolicy::Disabled));
    assert_eq!(JitPolicy::parse("vm"), Some(JitPolicy::Disabled));
    assert_eq!(JitPolicy::parse("invalid"), None);
}

#[test]
fn test_jit_policy_display_and_enabled() {
    assert!(JitPolicy::Auto.is_enabled());
    assert!(JitPolicy::Eager.is_enabled());
    assert!(!JitPolicy::Disabled.is_enabled());
    assert_eq!(format!("{}", JitPolicy::Auto), "JIT: Auto");
    assert_eq!(format!("{}", JitPolicy::Eager), "JIT: Eager");
    assert_eq!(format!("{}", JitPolicy::Disabled), "JIT: Off (VM)");
}
