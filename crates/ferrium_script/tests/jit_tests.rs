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
    assert!(summary.jit_kind.is_some());
    assert!(check_jit_eligibility(&artifact.chunk, 0).is_some());
}

#[test]
fn test_jit_eager_matches_vm_interpreter() {
    let src = r#"
        nya tri(n: i64) -> i64 {
            let mut s = 0;
            let mut i = 1;
            while i <= n {
                s = s + i;
                i = i + 1;
            }
            return s;
        }
        return tri(100);
    "#;
    let mut comp_vm = ScriptComponent::from_source_with_jit("vm.fy", src, JitPolicy::Disabled).unwrap();
    let mut comp_jit = ScriptComponent::from_source_with_jit("jit.fy", src, JitPolicy::Eager).unwrap();

    let mut host_vm = FerriumHost::new(ScriptSnapshot::default());
    let mut host_jit = FerriumHost::new(ScriptSnapshot::default());

    let r_vm = comp_vm.step(&mut host_vm).unwrap();
    let r_jit = comp_jit.step(&mut host_jit).unwrap();
    assert_eq!(format!("{:?}", r_vm), format!("{:?}", r_jit));
    assert!(matches!(r_jit, VmResult::Done(v) if v.to_string() == "5050"));
}

#[test]
fn test_jit_disabled_performs_no_compilation() {
    let src = "nya sq(x: i64) -> i64 { return x * x; }\nreturn sq(2) + sq(3) + sq(4) + sq(5);";
    let mut comp = ScriptComponent::from_source_with_jit("no_jit.fy", src, JitPolicy::Disabled).unwrap();
    let mut host = FerriumHost::new(ScriptSnapshot::default());
    let res = comp.step(&mut host).unwrap();
    assert!(matches!(res, VmResult::Done(v) if v.to_string() == "54"));
    let tel = comp.jit_telemetry();
    assert_eq!(tel.compiled_functions, 0);
    assert_eq!(tel.native_calls, 0);
}

#[test]
fn test_jit_auto_hot_threshold() {
    let src = "nya inc(x: i64) -> i64 { return x + 1; }\nlet mut a = 0;\na = inc(a);\na = inc(a);\na = inc(a);\na = inc(a);\na = inc(a);\nreturn a;";
    let mut comp = ScriptComponent::from_source_with_jit("hot.fy", src, JitPolicy::Auto).unwrap();
    let mut host = FerriumHost::new(ScriptSnapshot::default());
    let res = comp.step(&mut host).unwrap();
    assert!(matches!(res, VmResult::Done(v) if v.to_string() == "5"));
    if is_supported_platform() {
        let tel = comp.jit_telemetry();
        assert!(tel.compiled_functions >= 1);
        assert!(tel.native_calls >= 1);
    }
}

#[test]
fn test_dynamic_set_jit_policy() {
    let mut comp = ScriptComponent::from_source("dyn.fy", "return 1;").unwrap();
    assert_eq!(comp.jit_policy, JitPolicy::Auto);
    comp.set_jit_policy(JitPolicy::Eager);
    assert_eq!(comp.jit_policy, JitPolicy::Eager);
    comp.set_jit_policy(JitPolicy::Disabled);
    assert_eq!(comp.jit_policy, JitPolicy::Disabled);
}

#[test]
fn test_compile_artifact_function_summaries() {
    let src = r#"
        nya alpha(a: i64) -> i64 { return a + 1; }
        nya beta(a: i64, b: i64) -> i64 { return a + b; }
        return beta(alpha(2), 5);
    "#;
    let artifact = compile_fusty_detailed(src).unwrap();
    assert_eq!(artifact.functions.len(), 2);
    assert_eq!(artifact.functions[0].name, "alpha");
    assert_eq!(artifact.functions[0].arity, 1);
    assert_eq!(artifact.functions[1].name, "beta");
    assert_eq!(artifact.functions[1].arity, 2);
}
