use std::fmt;

use fusty_bytecode::chunk::Chunk;
pub use fusty_jit::{
    is_supported_platform, JitFallbackDiagnostic, JitMode, JitStats, JitValueKind,
};

/// High-level JIT policy exposed in Ferrium Engine CLI, Editor UI, and `ScriptComponent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JitPolicy {
    /// Compile hot `nya` functions automatically after 3 calls (default).
    #[default]
    Auto,
    /// Compile eligible `nya` functions immediately on first invocation.
    Eager,
    /// Run strictly in the bytecode VM interpreter without native compilation.
    Disabled,
}

impl JitPolicy {
    pub const DEFAULT_HOT_THRESHOLD: usize = 3;

    /// Converts this policy into `fusty_jit::JitMode`.
    #[inline]
    pub const fn to_jit_mode(self) -> JitMode {
        match self {
            Self::Auto => JitMode::Auto {
                hot_threshold: Self::DEFAULT_HOT_THRESHOLD,
            },
            Self::Eager => JitMode::Eager,
            Self::Disabled => JitMode::Disabled,
        }
    }

    /// Returns `true` if native JIT compilation is enabled under this policy.
    #[inline]
    pub const fn is_enabled(self) -> bool {
        !matches!(self, Self::Disabled)
    }

    /// Cycles through `Auto -> Eager -> Disabled -> Auto` (handy for toolbar buttons).
    #[inline]
    pub const fn cycle(self) -> Self {
        match self {
            Self::Auto => Self::Eager,
            Self::Eager => Self::Disabled,
            Self::Disabled => Self::Auto,
        }
    }

    /// Short human-readable label for editor badges and CLI telemetry.
    #[inline]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Auto => "JIT: Auto",
            Self::Eager => "JIT: Eager",
            Self::Disabled => "JIT: Off (VM)",
        }
    }

    /// Parses a policy from a CLI string (`auto`, `eager`, `off`/`disabled`/`vm`).
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "eager" | "always" | "on" => Some(Self::Eager),
            "off" | "disabled" | "none" | "vm" => Some(Self::Disabled),
            _ => None,
        }
    }
}

impl fmt::Display for JitPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Aggregated JIT runtime telemetry and silent fallback diagnostics across Ferrium script VMs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JitTelemetry {
    pub compiled_functions: usize,
    pub native_calls: usize,
    pub vm_fallbacks: usize,
    pub compiled_bytes: usize,
    pub compiled_names: Vec<String>,
    pub fallback_diagnostics: Vec<JitFallbackDiagnostic>,
    pub platform_supported: bool,
}

impl JitTelemetry {
    pub fn from_stats(stats: &JitStats) -> Self {
        Self {
            compiled_functions: stats.compiled_functions,
            native_calls: stats.native_calls,
            vm_fallbacks: stats.vm_fallbacks,
            compiled_bytes: stats.compiled_bytes,
            compiled_names: stats.compiled_names.clone(),
            fallback_diagnostics: stats.fallback_diagnostics.clone(),
            platform_supported: is_supported_platform(),
        }
    }

    /// Merges another VM's `JitStats` into this aggregate snapshot.
    pub fn accumulate(&mut self, stats: &JitStats) {
        self.platform_supported = is_supported_platform();
        self.compiled_functions += stats.compiled_functions;
        self.native_calls += stats.native_calls;
        self.vm_fallbacks += stats.vm_fallbacks;
        self.compiled_bytes += stats.compiled_bytes;
        for name in &stats.compiled_names {
            if !self.compiled_names.contains(name) {
                self.compiled_names.push(name.clone());
            }
        }
        for diag in &stats.fallback_diagnostics {
            if let Some(existing) = self.fallback_diagnostics.iter_mut().find(|d| {
                d.function == diag.function && d.kind == diag.kind && d.reason == diag.reason
            }) {
                existing.count += diag.count;
            } else {
                self.fallback_diagnostics.push(diag.clone());
            }
        }
    }

    /// Fraction of JIT-attempted calls executed natively (`0.0..=1.0`).
    pub fn native_hit_rate(&self) -> f64 {
        let total = self.native_calls + self.vm_fallbacks;
        if total == 0 {
            0.0
        } else {
            self.native_calls as f64 / total as f64
        }
    }
}

/// Analyzes whether `chunk.functions[fn_idx]` is eligible for x86-64 JIT compilation.
/// Checks `Int64` specialization first, then `Float64`.
pub fn check_jit_eligibility(chunk: &Chunk, fn_idx: usize) -> Option<JitValueKind> {
    if fusty_jit::analyze::analyze_function(chunk, fn_idx, JitValueKind::Int64).is_ok() {
        return Some(JitValueKind::Int64);
    }
    if fusty_jit::analyze::analyze_function(chunk, fn_idx, JitValueKind::Float64).is_ok() {
        return Some(JitValueKind::Float64);
    }
    None
}
