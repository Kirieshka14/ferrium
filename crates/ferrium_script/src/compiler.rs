use fusty_bytecode::chunk::Chunk;
use fusty_bytecode::compiler::Compiler;
use fusty_lexer::Lexer;
use fusty_parser::Parser;

use crate::error::{CompileStage, ScriptError};
use crate::jit::{check_jit_eligibility, JitValueKind};

/// Metadata about a compiled FustyLang `nya` function, including native JIT eligibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSummary {
    pub name: String,
    pub arity: u8,
    pub entry_ip: usize,
    pub end_ip: usize,
    pub is_meow: bool,
    pub jit_kind: Option<JitValueKind>,
}

/// Complete compilation output including bytecode chunk, token count, and function summaries.
#[derive(Debug, Clone)]
pub struct CompileArtifact {
    pub chunk: Chunk,
    pub token_count: usize,
    pub functions: Vec<FunctionSummary>,
}

impl CompileArtifact {
    /// Number of `nya` functions in this script eligible for native x86-64 JIT compilation.
    pub fn jit_eligible_count(&self) -> usize {
        self.functions.iter().filter(|f| f.jit_kind.is_some()).count()
    }
}

/// Compiles FustyLang 0.2.1 source code through Lexer -> Parser -> Bytecode Compiler
/// and inspects every `nya` function for native `fusty_jit` eligibility.
pub fn compile_fusty_detailed(source: &str) -> Result<CompileArtifact, ScriptError> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|(msg, span)| ScriptError::Compile {
            stage: CompileStage::Lexer,
            line: span.line,
            col: span.col,
            message: msg,
        })?;
    let token_count = tokens.len();

    let program = Parser::new(tokens)
        .parse_program()
        .map_err(|e| ScriptError::Compile {
            stage: CompileStage::Parser,
            line: e.span.line,
            col: e.span.col,
            message: e.message,
        })?;

    let chunk = Compiler::new()
        .compile_program(&program)
        .map_err(|e| ScriptError::Compile {
            stage: CompileStage::Bytecode,
            line: 0,
            col: 0,
            message: e.message,
        })?;

    let functions = chunk
        .functions
        .iter()
        .enumerate()
        .map(|(idx, f)| FunctionSummary {
            name: f.name.clone(),
            arity: f.arity,
            entry_ip: f.entry_ip,
            end_ip: f.end_ip,
            is_meow: f.is_meow,
            jit_kind: check_jit_eligibility(&chunk, idx),
        })
        .collect();

    Ok(CompileArtifact {
        chunk,
        token_count,
        functions,
    })
}

/// Compiles FustyLang 0.2.1 source code into a bytecode `Chunk`.
pub fn compile_fusty_source(source: &str) -> Result<Chunk, String> {
    compile_fusty_detailed(source)
        .map(|artifact| artifact.chunk)
        .map_err(|e| e.to_string())
}
