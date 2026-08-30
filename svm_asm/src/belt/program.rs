use super::error::AsmError;
use std::{fs, path::Path};

const HEADER_SIZE: usize = 12;
pub const EXECUTABLE_MAGIC: &[u8; 4] = b"SVB\x01";
const MEMORY_SIZE: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub load_address: u16,
    pub entry_address: u16,
    pub payload: Vec<u8>,
}

impl Program {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AsmError> {
        if bytes.len() < HEADER_SIZE {
            return Err(AsmError::ProgramFormat(
                "file is shorter than the 12-byte header".into(),
            ));
        }
        if &bytes[..4] != EXECUTABLE_MAGIC {
            return Err(AsmError::ProgramFormat(
                "invalid or incompatible SVM executable magic".into(),
            ));
        }
        let load_address = u16::from_le_bytes([bytes[4], bytes[5]]);
        let entry_address = u16::from_le_bytes([bytes[6], bytes[7]]);
        let payload_size = usize::try_from(u32::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11],
        ]))
        .map_err(|_| AsmError::ProgramFormat("payload size does not fit host usize".into()))?;
        let end = HEADER_SIZE
            .checked_add(payload_size)
            .ok_or_else(|| AsmError::ProgramFormat("payload size overflow".into()))?;
        if end != bytes.len() {
            return Err(AsmError::ProgramFormat(
                "payload length does not match header".into(),
            ));
        }
        validate_load_range(load_address, payload_size)?;
        Ok(Self {
            load_address,
            entry_address,
            payload: bytes[HEADER_SIZE..].to_vec(),
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, AsmError> {
        validate_load_range(self.load_address, self.payload.len())?;
        let payload_size = u32::try_from(self.payload.len())
            .map_err(|_| AsmError::ProgramFormat("payload is too large".into()))?;
        let mut bytes = Vec::with_capacity(HEADER_SIZE + self.payload.len());
        bytes.extend_from_slice(EXECUTABLE_MAGIC);
        bytes.extend_from_slice(&self.load_address.to_le_bytes());
        bytes.extend_from_slice(&self.entry_address.to_le_bytes());
        bytes.extend_from_slice(&payload_size.to_le_bytes());
        bytes.extend_from_slice(&self.payload);
        Ok(bytes)
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), AsmError> {
        fs::write(path, self.to_bytes()?)?;
        Ok(())
    }
}

fn validate_load_range(load_address: u16, payload_size: usize) -> Result<(), AsmError> {
    let end = (load_address as usize)
        .checked_add(payload_size)
        .ok_or_else(|| AsmError::ProgramFormat("load range overflow".into()))?;
    if end > MEMORY_SIZE {
        return Err(AsmError::ProgramFormat(
            "program does not fit into 64 KiB memory".into(),
        ));
    }
    Ok(())
}
