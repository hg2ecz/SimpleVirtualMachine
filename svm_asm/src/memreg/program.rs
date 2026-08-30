use super::error::AsmError;
use std::{fs, path::Path};
pub const EXECUTABLE_MAGIC: &[u8; 4] = b"SVF\x04";
const HEADER: usize = 12;
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
        let mut o = Vec::with_capacity(HEADER + self.payload.len());
        o.extend_from_slice(EXECUTABLE_MAGIC);
        o.extend_from_slice(&self.load_address.to_le_bytes());
        o.extend_from_slice(&self.entry_address.to_le_bytes());
        o.extend_from_slice(&n.to_le_bytes());
        o.extend_from_slice(&self.payload);
        Ok(o)
    }
    pub fn write_file(&self, p: impl AsRef<Path>) -> Result<(), AsmError> {
        fs::write(p, self.to_bytes()?)?;
        Ok(())
    }
}
