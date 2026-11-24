use std::fs;
use std::path::{Path, PathBuf};

use fusty_bytecode::chunk::{Chunk, Constant};
use fusty_bytecode::compiler::Compiler;
use fusty_bytecode::opcode::Opcode;
use fusty_lexer::Lexer;
use fusty_parser::Parser;
use fusty_vm::value::Value;
use fusty_vm::vm::{Host, Vm, VmResult};

#[derive(Debug, Clone, Default)]
pub struct CompileReport {
    pub ok: bool,
    pub summary: String,
    pub error: Option<String>,
    pub functions: Vec<(String, u8, usize, bool)>,
    pub disassembly: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OpenScriptTab {
    pub path: Option<PathBuf>,
    pub title: String,
    pub buffer: String,
    pub dirty: bool,
    pub last_check: CompileReport,
    pub cursor_line: usize,
    pub cursor_col: usize,
    cached_line_numbers: String,
    cached_line_count: usize,
}

impl OpenScriptTab {
    pub fn compile_check(&mut self) {
        self.last_check = analyze_source(&self.buffer);
        self.refresh_line_numbers();
    }

    pub fn refresh_line_numbers(&mut self) {
        let count = self.buffer.lines().count().max(1);
        if count != self.cached_line_count || self.cached_line_numbers.is_empty() {
            self.cached_line_count = count;
            self.cached_line_numbers = (1..=count)
                .map(|n| format!("{:>2}", n))
                .collect::<Vec<_>>()
                .join("\n");
        }
    }

    pub fn update_cursor(&mut self, char_index: usize) {
        let mut line = 1usize;
        let mut col = 1usize;
        for (idx, ch) in self.buffer.chars().enumerate() {
            if idx >= char_index {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        self.cursor_line = line;
        self.cursor_col = col;
    }

    #[inline]
    pub fn line_numbers_text(&self) -> &str {
        &self.cached_line_numbers
    }
}

pub struct CodeEditorState {
    pub tabs: Vec<OpenScriptTab>,
    pub active_tab: usize,
    pub show_bytecode: bool,
}

impl CodeEditorState {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
            show_bytecode: false,
        }
    }

    pub fn active(&self) -> Option<&OpenScriptTab> {
        self.tabs.get(self.active_tab)
    }

    pub fn active_mut(&mut self) -> Option<&mut OpenScriptTab> {
        self.tabs.get_mut(self.active_tab)
    }

    pub fn open_file(&mut self, path: &Path) -> Result<(), String> {
        if let Some(existing) = self
            .tabs
            .iter()
            .position(|t| t.path.as_deref() == Some(path))
        {
            self.active_tab = existing;
            return Ok(());
        }
        let src = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read '{}': {}", path.display(), e))?;
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("script.fy")
            .to_string();
        let mut tab = OpenScriptTab {
            path: Some(path.to_path_buf()),
            title,
            buffer: src,
            dirty: false,
            last_check: CompileReport::default(),
            cursor_line: 1,
            cursor_col: 1,
            cached_line_numbers: String::new(),
            cached_line_count: 0,
        };
        tab.compile_check();
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        Ok(())
    }

    pub fn save_active(&mut self) -> Result<PathBuf, String> {
        let tab = self
            .tabs
            .get_mut(self.active_tab)
            .ok_or_else(|| "No active script tab".to_string())?;
        let path = tab
            .path
            .clone()
            .ok_or_else(|| "No file path assigned to tab".to_string())?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create '{}': {}", parent.display(), e))?;
        }
        fs::write(&path, &tab.buffer)
            .map_err(|e| format!("Failed to write '{}': {}", path.display(), e))?;
        tab.dirty = false;
        tab.compile_check();
        Ok(path)
    }

    pub fn create_new_script(&mut self, dir: &Path, name: &str) -> Result<PathBuf, String> {
        let sanitized = name.trim().trim_end_matches(".fy");
        if sanitized.is_empty() {
            return Err("Script name cannot be empty".into());
        }
        fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {}", e))?;
        let path = dir.join(format!("{}.fy", sanitized));
        let template = format!(
            "// Ferrium FustyLang — {name}.fy\n\nlet speed = 4.0;\n\nfn update() {{\n    let axis = vec2(input_x(), input_y());\n    set_self_vel(axis * speed);\n}}\n\nwhile true {{\n    update();\n    wait 0.016;\n}}\n",
            name = sanitized
        );
        fs::write(&path, &template).map_err(|e| format!("Write error: {}", e))?;
        let mut tab = OpenScriptTab {
            path: Some(path.clone()),
            title: format!("{}.fy", sanitized),
            buffer: template,
            dirty: false,
            last_check: CompileReport::default(),
            cursor_line: 1,
            cursor_col: 1,
            cached_line_numbers: String::new(),
            cached_line_count: 0,
        };
        tab.compile_check();
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        Ok(path)
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() && self.tabs.len() > 1 {
            self.tabs.remove(idx);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }
}

pub fn analyze_source(source: &str) -> CompileReport {
    let tokens = match Lexer::new(source).tokenize() {
        Ok(t) => t,
        Err((msg, span)) => {
            return CompileReport {
                ok: false,
                summary: format!("Lexer error {}:{}", span.line, span.col),
                error: Some(format!("Line {}:{}: {}", span.line, span.col, msg)),
                ..Default::default()
            };
        }
    };
    let tok_count = tokens.len();

    let program = match Parser::new(tokens).parse_program() {
        Ok(p) => p,
        Err(e) => {
            return CompileReport {
                ok: false,
                summary: format!("Parser error {}:{}", e.span.line, e.span.col),
                error: Some(format!("Line {}:{}: {}", e.span.line, e.span.col, e.message)),
                ..Default::default()
            };
        }
    };

    let chunk = match Compiler::new().compile_program(&program) {
        Ok(c) => c,
        Err(e) => {
            return CompileReport {
                ok: false,
                summary: "Compile error".into(),
                error: Some(e.message),
                ..Default::default()
            };
        }
    };

    let functions = chunk
        .functions
        .iter()
        .map(|f| (f.name.clone(), f.arity, f.entry_ip, f.is_meow))
        .collect();
    let disassembly = disassemble_chunk(&chunk);

    CompileReport {
        ok: true,
        summary: format!(
            "No errors ({}B bytecode, {} tok)",
            chunk.code.len(),
            tok_count
        ),
        error: None,
        functions,
        disassembly,
    }
}

pub fn disassemble_chunk(chunk: &Chunk) -> Vec<String> {
    let mut lines = Vec::new();
    let mut ip = 0usize;
    while ip < chunk.code.len() {
        let addr = ip;
        let byte = chunk.code[ip];
        ip += 1;
        let src_line = chunk.lines.get(addr).copied().unwrap_or(0);
        let Some(op) = Opcode::from_u8(byte) else {
            lines.push(format!("{:04} [L{:>3}] DB 0x{:02X}", addr, src_line, byte));
            continue;
        };

        match op {
            Opcode::Const => {
                let idx = read_u16(&chunk.code, &mut ip);
                let c_str = chunk
                    .constants
                    .get(idx as usize)
                    .map(format_const)
                    .unwrap_or_else(|| "?".into());
                lines.push(format!(
                    "{:04} [L{:>3}] Const #{} ({})",
                    addr, src_line, idx, c_str
                ));
            }
            Opcode::PopUnder
            | Opcode::GetLocal
            | Opcode::SetLocal
            | Opcode::GetGlobal
            | Opcode::SetGlobal
            | Opcode::Jump
            | Opcode::JumpIfFalse
            | Opcode::Loop => {
                let arg = read_u16(&chunk.code, &mut ip);
                lines.push(format!("{:04} [L{:>3}] {:?} {}", addr, src_line, op, arg));
            }
            Opcode::Call => {
                let fn_idx = read_u16(&chunk.code, &mut ip);
                let argc = read_u8(&chunk.code, &mut ip);
                let fn_name = chunk
                    .functions
                    .get(fn_idx as usize)
                    .map(|f| f.name.as_str())
                    .unwrap_or("?");
                lines.push(format!(
                    "{:04} [L{:>3}] Call {} (#{}, argc={})",
                    addr, src_line, fn_name, fn_idx, argc
                ));
            }
            Opcode::CallNative => {
                let name_idx = read_u16(&chunk.code, &mut ip);
                let argc = read_u8(&chunk.code, &mut ip);
                let name = match chunk.constants.get(name_idx as usize) {
                    Some(Constant::String(s)) => s.as_str(),
                    _ => "?",
                };
                lines.push(format!(
                    "{:04} [L{:>3}] CallNative '{}' (argc={})",
                    addr, src_line, name, argc
                ));
            }
            _ => {
                lines.push(format!("{:04} [L{:>3}] {:?}", addr, src_line, op));
            }
        }
    }
    lines
}

fn read_u8(code: &[u8], ip: &mut usize) -> u8 {
    let b = code.get(*ip).copied().unwrap_or(0);
    *ip += 1;
    b
}

fn read_u16(code: &[u8], ip: &mut usize) -> u16 {
    let hi = read_u8(code, ip) as u16;
    let lo = read_u8(code, ip) as u16;
    (hi << 8) | lo
}

fn format_const(c: &Constant) -> String {
    match c {
        Constant::Int(i) => i.to_string(),
        Constant::Float(f) => format!("{:.2}", f),
        Constant::Bool(b) => b.to_string(),
        Constant::String(s) => format!("\"{}\"", s),
        Constant::Vec2(x, y) => format!("vec2({:.1}, {:.1})", x, y),
        Constant::Vec3(x, y, z) => format!("vec3({:.1}, {:.1}, {:.1})", x, y, z),
    }
}

struct ReplHost {
    output: Vec<String>,
}

impl Host for ReplHost {
    fn call_native(&mut self, name: &str, args: &[Value], meow_mode: bool) -> Result<Value, String> {
        match name {
            "log" => {
                let text = args
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                self.output.push(text);
                Ok(Value::Nil)
            }
            "sin" => args
                .first()
                .and_then(|v| v.as_f64())
                .map(|x| Value::Float(x.sin()))
                .ok_or_else(|| "sin expects number".into()),
            "cos" => args
                .first()
                .and_then(|v| v.as_f64())
                .map(|x| Value::Float(x.cos()))
                .ok_or_else(|| "cos expects number".into()),
            "sqrt" => args
                .first()
                .and_then(|v| v.as_f64())
                .map(|x| Value::Float(x.max(0.0).sqrt()))
                .ok_or_else(|| "sqrt expects number".into()),
            "abs" => args
                .first()
                .and_then(|v| v.as_f64())
                .map(|x| Value::Float(x.abs()))
                .ok_or_else(|| "abs expects number".into()),
            "min" if args.len() == 2 => {
                let a = args[0].as_f64().ok_or("min expects numbers")?;
                let b = args[1].as_f64().ok_or("min expects numbers")?;
                Ok(Value::Float(a.min(b)))
            }
            "max" if args.len() == 2 => {
                let a = args[0].as_f64().ok_or("max expects numbers")?;
                let b = args[1].as_f64().ok_or("max expects numbers")?;
                Ok(Value::Float(a.max(b)))
            }
            "clamp" if args.len() == 3 => {
                let x = args[0].as_f64().ok_or("clamp expects numbers")?;
                let lo = args[1].as_f64().ok_or("clamp expects numbers")?;
                let hi = args[2].as_f64().ok_or("clamp expects numbers")?;
                Ok(Value::Float(x.clamp(lo, hi)))
            }
            "lerp" if args.len() == 3 => {
                let a = args[0].as_f64().ok_or("lerp expects numbers")?;
                let b = args[1].as_f64().ok_or("lerp expects numbers")?;
                let t = args[2].as_f64().ok_or("lerp expects numbers")?;
                Ok(Value::Float(a + (b - a) * t))
            }
            "is_meow" => Ok(Value::Bool(meow_mode)),
            other => Err(format!("Native '{}' is only available in Play mode", other)),
        }
    }
}

pub struct ReplState {
    pub snippet: String,
    pub output: Vec<String>,
    pub last_result: Option<String>,
    pub last_error: Option<String>,
}

impl Default for ReplState {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplState {
    pub fn new() -> Self {
        Self {
            snippet: "let a = vec2(30.0, 40.0);\nlog(\"len =\", a.length());\nreturn a.normalized() * 100.0;".into(),
            output: Vec::new(),
            last_result: None,
            last_error: None,
        }
    }

    pub fn run_snippet(&mut self) {
        self.output.clear();
        self.last_result = None;
        self.last_error = None;

        let chunk = match ferrium_script::compile_fusty_source(&self.snippet) {
            Ok(c) => c,
            Err(e) => {
                self.last_error = Some(e);
                return;
            }
        };

        let mut vm = Vm::new();
        let mut host = ReplHost { output: Vec::new() };
        for _ in 0..16 {
            match vm.run_with_host(&chunk, &mut host) {
                Ok(VmResult::Done(val)) => {
                    self.output = host.output;
                    self.last_result = Some(val.to_string());
                    return;
                }
                Ok(VmResult::Waiting { duration_sec }) => {
                    host.output.push(format!("[wait {:.2}s]", duration_sec));
                }
                Ok(VmResult::Yielded) => {
                    host.output.push("[yield]".into());
                    self.output = host.output;
                    return;
                }
                Err(e) => {
                    self.output = host.output;
                    self.last_error = Some(format!("VM line {}: {}", e.line, e.message));
                    return;
                }
            }
        }
        self.output = host.output;
        self.last_error = Some("REPL wait limit reached".into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_fusty_and_disassemble() {
        let rep = analyze_source("fn helper(x: i32) -> i32 { return x * 2; }\nlet v = helper(21);");
        assert!(rep.ok, "{:?}", rep.error);
        assert_eq!(rep.functions.len(), 1);
        assert!(!rep.disassembly.is_empty());
    }

    #[test]
    fn test_new_script_template_compiles_with_globals() {
        let template = "let speed = 4.0;\nfn update() {\n    let axis = vec2(input_x(), input_y());\n    set_self_vel(axis * speed);\n}\nwhile true { update(); wait 0.016; }\n";
        let rep = analyze_source(template);
        assert!(rep.ok, "New script template failed: {:?}", rep.error);
    }

    #[test]
    fn test_repl_executes_fusty_snippet() {
        let mut repl = ReplState::new();
        repl.run_snippet();
        assert!(repl.last_error.is_none(), "{:?}", repl.last_error);
        assert_eq!(repl.last_result.as_deref(), Some("vec2(60.00, 80.00)"));
    }
}
