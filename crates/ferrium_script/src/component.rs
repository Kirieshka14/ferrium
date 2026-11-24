use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use fusty_bytecode::chunk::Chunk;
use fusty_vm::vm::{Vm, VmResult};

use crate::compiler::compile_fusty_source;
use crate::error::ScriptError;
use crate::host::FerriumHost;
use crate::jit::{JitPolicy, JitTelemetry};

/// ECS component holding a compiled FustyLang 0.2.1 VM, native `fusty_jit` policy,
/// coroutine timer, safe `baka!` trap telemetry, and hot-reload state.
pub struct ScriptComponent {
    pub path: PathBuf,
    pub chunk: Chunk,
    pub vm: Vm,
    pub jit_policy: JitPolicy,
    pub wait_remaining: f64,
    pub last_mtime: Option<SystemTime>,
    pub poll_timer: f64,
    pub reload_count: u32,
    pub last_error: Option<String>,
    pub last_baka: Option<String>,
    pub baka_count: u32,
    pub purr_count: u32,
    pub status: String,
    pub meow_flash_timer: f32,
}

impl ScriptComponent {
    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self, String> {
        Self::from_file_with_jit(path, JitPolicy::default())
    }

    pub fn from_file_with_jit(
        path: impl Into<PathBuf>,
        jit_policy: JitPolicy,
    ) -> Result<Self, String> {
        let path = path.into();
        let source = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read '{}': {}", path.display(), e))?;
        let mtime = fs::metadata(&path).and_then(|m| m.modified()).ok();
        let mut comp = Self::from_source_with_jit(path, &source, jit_policy)?;
        comp.last_mtime = mtime;
        Ok(comp)
    }

    pub fn from_source(path: impl Into<PathBuf>, source: &str) -> Result<Self, String> {
        Self::from_source_with_jit(path, source, JitPolicy::default())
    }

    pub fn from_source_with_jit(
        path: impl Into<PathBuf>,
        source: &str,
        jit_policy: JitPolicy,
    ) -> Result<Self, String> {
        let chunk = compile_fusty_source(source)?;
        Ok(Self {
            path: path.into(),
            chunk,
            vm: Vm::with_jit_mode(jit_policy.to_jit_mode()),
            jit_policy,
            wait_remaining: 0.0,
            last_mtime: None,
            poll_timer: 0.0,
            reload_count: 0,
            last_error: None,
            last_baka: None,
            baka_count: 0,
            purr_count: 0,
            status: "ready".into(),
            meow_flash_timer: 0.0,
        })
    }

    /// Updates the native `fusty_jit` policy on the underlying VM.
    pub fn set_jit_policy(&mut self, policy: JitPolicy) {
        self.jit_policy = policy;
        self.vm.set_jit_mode(policy.to_jit_mode());
    }

    /// Returns a snapshot of this script VM's native JIT compilation & execution stats.
    pub fn jit_telemetry(&self) -> JitTelemetry {
        JitTelemetry::from_stats(self.vm.jit_stats())
    }

    /// Returns whether the controllable FustyLang GC is enabled (enabled by default).
    #[inline]
    pub fn gc_is_enabled(&self) -> bool {
        self.vm.gc_is_enabled()
    }

    /// Enables or disables automatic GC for this script VM.
    #[inline]
    pub fn set_gc_enabled(&mut self, enabled: bool) {
        self.vm.set_gc_enabled(enabled);
    }

    /// Runs a full mark-and-sweep GC cycle and returns the number of freed objects.
    #[inline]
    pub fn gc_collect(&mut self) -> usize {
        self.vm.gc_collect()
    }

    /// Runs a budgeted incremental GC step and returns the number of freed objects.
    #[inline]
    pub fn gc_step(&mut self, budget: usize) -> usize {
        self.vm.gc_step(budget)
    }

    /// Returns a snapshot of this script VM's GC telemetry.
    #[inline]
    pub fn gc_stats(&self) -> fusty_vm::gc::GcStats {
        self.vm.gc_stats()
    }

    /// Advances the FustyLang VM one step with safe `baka!(...)` interception and telemetry updates.
    pub fn step(&mut self, host: &mut FerriumHost) -> Result<VmResult, ScriptError> {
        let result = self.vm.run_with_host(&self.chunk, host);
        self.purr_count += host.purr_count;
        match result {
            Ok(res) => {
                match &res {
                    VmResult::Waiting { duration_sec } => {
                        self.wait_remaining = (*duration_sec).max(0.0);
                        self.status = "waiting".into();
                    }
                    VmResult::Yielded => {
                        self.status = "yielded".into();
                    }
                    VmResult::Done(_) => {
                        self.status = "done".into();
                    }
                }
                Ok(res)
            }
            Err(vm_err) => {
                let err = ScriptError::runtime(vm_err.line, vm_err.message);
                let msg = err.to_string();
                self.last_error = Some(msg.clone());
                if err.is_baka() {
                    self.baka_count += 1;
                    self.last_baka = Some(msg);
                    self.status = "baka!".into();
                } else {
                    self.status = "error".into();
                }
                Err(err)
            }
        }
    }

    /// Recovers a script cleanly from a `baka!(...)` trap or runtime error by resetting VM state
    /// while preserving the active `JitPolicy`.
    pub fn recover_from_baka(&mut self) {
        self.vm.reset_execution();
        self.vm.set_jit_mode(self.jit_policy.to_jit_mode());
        self.wait_remaining = 0.0;
        self.last_error = None;
        self.last_baka = None;
        self.status = "recovered".into();
    }

    /// Checks file mtime and hot-reloads bytecode on change while preserving the last valid
    /// script on compile error and keeping the active `JitPolicy`.
    pub fn try_hot_reload(&mut self, force: bool) -> Option<Result<u32, String>> {
        let mtime = fs::metadata(&self.path).and_then(|m| m.modified()).ok();
        let changed = match (self.last_mtime, mtime) {
            (Some(prev), Some(curr)) => curr != prev,
            (None, Some(_)) => true,
            _ => false,
        };
        if !force && !changed {
            return None;
        }
        if let Some(m) = mtime {
            self.last_mtime = Some(m);
        }

        let source = match fs::read_to_string(&self.path) {
            Ok(s) => s,
            Err(e) => {
                let msg = format!("Read error: {}", e);
                self.last_error = Some(msg.clone());
                return Some(Err(msg));
            }
        };

        match compile_fusty_source(&source) {
            Ok(new_chunk) => {
                self.chunk = new_chunk;
                self.vm.reset_execution();
                self.vm.set_jit_mode(self.jit_policy.to_jit_mode());
                self.wait_remaining = 0.0;
                self.reload_count += 1;
                self.last_error = None;
                self.last_baka = None;
                self.status = format!("reloaded #{}", self.reload_count);
                Some(Ok(self.reload_count))
            }
            Err(err) => {
                self.last_error = Some(err.clone());
                Some(Err(err))
            }
        }
    }
}
