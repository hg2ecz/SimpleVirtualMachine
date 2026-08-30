use super::error::AsmError;
use std::{fs, path::Path};

pub const EXECUTABLE_MAGIC: &[u8; 4] = b"SVA\x06";
const HEADER_SIZE: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub load_address: u16,
    pub entry_address: u16,
    pub payload: Vec<u8>,
}

impl Program {
    pub fn to_bytes(&self) -> Result<Vec<u8>, AsmError> {
        let n = u32::try_from(self.payload.len())
            .map_err(|_| AsmError::Program("payload too large".into()))?;
        if self.load_address as usize + self.payload.len() > 65_536 {
            return Err(AsmError::Program("program does not fit in 64 KiB".into()));
        }
        let mut out = Vec::with_capacity(HEADER_SIZE + self.payload.len());
        out.extend_from_slice(EXECUTABLE_MAGIC);
        out.extend_from_slice(&self.load_address.to_le_bytes());
        out.extend_from_slice(&self.entry_address.to_le_bytes());
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&self.payload);
        Ok(out)
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), AsmError> {
        fs::write(path, self.to_bytes()?)?;
        Ok(())
    }
}
