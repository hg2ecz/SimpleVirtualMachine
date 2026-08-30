use thiserror::Error;

#[derive(Debug, Error)]
pub enum AsmError {
    #[error("invalid executable format: {0}")]
    ProgramFormat(String),
    #[error("assembler error on line {line}: {message}")]
    Assembly { line: usize, message: String },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
