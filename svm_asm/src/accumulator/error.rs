use thiserror::Error;

#[derive(Debug, Error)]
pub enum AsmError {
    #[error("line {line}: {message}")]
    Syntax { line: usize, message: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("program format: {0}")]
    Program(String),
}

pub fn syntax(line: usize, message: impl Into<String>) -> AsmError {
    AsmError::Syntax {
        line,
        message: message.into(),
    }
}
