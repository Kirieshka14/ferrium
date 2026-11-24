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

#[test]
fn test_jit_telemetry_accumulate() {
    let mut tel = JitTelemetry::default();
    let s1 = JitStats {
        compiled_functions: 1,
        native_calls: 8,
        vm_fallbacks: 2,
        compiled_bytes: 128,
        compiled_names: vec!["fib".into()],
    };
    let s2 = JitStats {
        compiled_functions: 1,
        native_calls: 2,
        vm_fallbacks: 0,
        compiled_bytes: 64,
        compiled_names: vec!["fib".into(), "step".into()],
    };
    tel.accumulate(&s1);
    tel.accumulate(&s2);
    assert_eq!(tel.compiled_functions, 2);
    assert_eq!(tel.native_calls, 10);
    assert_eq!(tel.vm_fallbacks, 2);
    assert_eq!(tel.compiled_bytes, 192);
    assert_eq!(tel.compiled_names, vec!["fib".to_string(), "step".to_string()]);
}

#[test]
fn test_jit_telemetry_hit_rate() {
    let empty = JitTelemetry::default();
    assert_eq!(empty.native_hit_rate(), 0.0);

    let stats = JitStats {
        compiled_functions: 1,
        native_calls: 3,
        vm_fallbacks: 1,
        compiled_bytes: 64,
        compiled_names: vec!["calc".into()],
    };
    let tel = JitTelemetry::from_stats(&stats);
    assert!((tel.native_hit_rate() - 0.75).abs() < 1e-6);
    assert_eq!(tel.platform_supported, is_supported_platform());
}

#[test]
fn test_jit_eligibility_numeric_function() {
    let artifact = compile_fusty_detailed(
        "nya sum_to(n: i64) -> i64 { let mut acc = 0; let mut i = 1; while i <= n { acc = acc + i; i = i + 1; } return acc; }
return sum_to(10);",
    ).unwrap();
    assert_eq!(artifact.functions.len(), 1);
    let summary = &artifact.functions[0];
    assert_eq!(summary.name, "sum_to");
    assert_eq!(summary.arity, 1);
    assert!(summary.jit_eligible);
    assert!(check_jit_eligibility(&artifact.chunk, 0).is_ok());
}
