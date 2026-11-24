use std::fmt;

/// Stage of the FustyLang 0.2.1 compilation pipeline where an error occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileStage {
    Lexer,
    Parser,
    Bytecode,
}

impl fmt::Display for CompileStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lexer => write!(f, "Lexer"),
            Self::Parser => write!(f, "Parser"),
            Self::Bytecode => write!(f, "Bytecode"),
        }
    }
}

/// Structured error produced while loading, compiling, or executing a FustyLang script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptError {
    Io {
        path: String,
        message: String,
    },
    Compile {
        stage: CompileStage,
        line: usize,
        col: usize,
        message: String,
    },
    Runtime {
        line: usize,
        message: String,
        is_baka: bool,
    },
}

impl ScriptError {
    /// Returns `true` when this runtime error originated from a FustyLang `baka!(...)` panic.
    #[inline]
    pub fn is_baka(&self) -> bool {
        matches!(self, Self::Runtime { is_baka: true, .. })
    }

    /// Constructs a runtime error while automatically classifying `baka!(...)` traps.
    pub fn runtime(line: usize, message: impl Into<String>) -> Self {
        let message = message.into();
        let is_baka = is_baka_message(&message);
        Self::Runtime {
            line,
            message,
            is_baka,
        }
    }

    /// Returns the 1-based source line number if available.
    pub fn line(&self) -> Option<usize> {
        match self {
            Self::Compile { line, .. } if *line > 0 => Some(*line),
            Self::Runtime { line, .. } if *line > 0 => Some(*line),
            _ => None,
        }
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => write!(f, "Failed to read '{}': {}", path, message),
            Self::Compile {
                stage,
                line,
                col,
                message,
            } => {
                if *line > 0 {
                    write!(f, "{} {}:{}: {}", stage, line, col, message)
                } else {
                    write!(f, "{} error: {}", stage, message)
                }
            }
            Self::Runtime {
                line,
                message,
                is_baka: true,
            } => write!(f, "BAKA trap at line {}: {}", line, message),
            Self::Runtime {
                line,
                message,
                is_baka: false,
            } => write!(f, "VM line {}: {}", line, message),
        }
    }
}

impl std::error::Error for ScriptError {}

/// Detects whether a FustyLang VM error message was raised by `baka!(...)`.
#[inline]
pub fn is_baka_message(message: &str) -> bool {
    message.contains("[BAKA!") || message.contains("BAKA! >///<")
}
