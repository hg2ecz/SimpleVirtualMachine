use thiserror::Error;

#[derive(Debug, Error)]
pub enum AsmError {
    #[error("program format error: {0}")]
    ProgramFormat(String),
    #[error("assembler error: {0}")]
    Assembler(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
