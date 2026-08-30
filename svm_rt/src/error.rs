use thiserror::Error;

#[derive(Debug, Error)]
pub enum VmError {
    #[error("invalid opcode 0x{0:02X}")]
    InvalidOpcode(u8),
    #[error("invalid register index {0}")]
    InvalidRegister(u8),
    #[error("invalid register operand encoding 0x{0:02X}")]
    InvalidRegisterEncoding(u8),
    #[error("invalid {width}-byte memory access at 0x{address:04X}")]
    InvalidMemoryAccess { address: u16, width: u8 },
    #[error("invalid memory range: start=0x{address:04X}, length={length}")]
    InvalidMemoryRange { address: u16, length: usize },
    #[error("division by zero")]
    DivisionByZero,
    #[error("stack overflow")]
    StackOverflow,
    #[error("stack underflow")]
    StackUnderflow,
    #[error("data stack underflow")]
    DataStackUnderflow,
    #[error("data stack overflow")]
    DataStackOverflow,
    #[error("return stack underflow")]
    ReturnStackUnderflow,
    #[error("return stack overflow")]
    ReturnStackOverflow,
    #[error("invalid video output buffer length: expected {expected}, got {actual}")]
    InvalidVideoBufferSize { expected: usize, actual: usize },
    #[error("program format error: {0}")]
    ProgramFormat(String),
    #[error("assembler error: {0}")]
    Assembler(String),
    #[error("assembler error on line {line}: {message}")]
    Assembly { line: usize, message: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
