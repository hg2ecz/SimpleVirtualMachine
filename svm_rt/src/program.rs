use crate::{
    error::VmError,
    memory::{MEMORY_SIZE, Memory},
};
use std::{fs, path::Path};

const HEADER_SIZE: usize = 12;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuKind {
    Register,
    Stack,
    Accumulator,
    Memreg,
    LoadStore,
    RegMem,
    Memory2Memory,
    Belt,
    Tta,
}

impl CpuKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Register => "register",
            Self::Stack => "stack",
            Self::Accumulator => "accumulator",
            Self::Memreg => "memreg",
            Self::LoadStore => "loadstore",
            Self::RegMem => "regmem",
            Self::Memory2Memory => "memory2memory",
            Self::Belt => "belt",
            Self::Tta => "tta",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub cpu: CpuKind,
    pub load_address: u16,
    pub entry_address: u16,
    pub payload: Vec<u8>,
}

impl Program {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, VmError> {
        if bytes.len() < HEADER_SIZE {
            return Err(VmError::ProgramFormat(
                "file is shorter than 12-byte header".into(),
            ));
        }
        let magic = &bytes[..4];
        let cpu = if magic == &svm_asm::register::program::EXECUTABLE_MAGIC[..] {
            CpuKind::Register
        } else if magic == &svm_asm::stack::program::EXECUTABLE_MAGIC[..] {
            CpuKind::Stack
        } else if magic == &svm_asm::accumulator::program::EXECUTABLE_MAGIC[..] {
            CpuKind::Accumulator
        } else if magic == &svm_asm::memreg::program::EXECUTABLE_MAGIC[..] {
            CpuKind::Memreg
        } else if magic == &svm_asm::loadstore::program::EXECUTABLE_MAGIC[..] {
            CpuKind::LoadStore
        } else if magic == &svm_asm::regmem::program::EXECUTABLE_MAGIC[..] {
            CpuKind::RegMem
        } else if magic == &svm_asm::memory2memory::program::EXECUTABLE_MAGIC[..] {
            CpuKind::Memory2Memory
        } else if magic == &svm_asm::belt::program::EXECUTABLE_MAGIC[..] {
            CpuKind::Belt
        } else if magic == &svm_asm::tta::program::EXECUTABLE_MAGIC[..] {
            CpuKind::Tta
        } else {
            return Err(VmError::ProgramFormat(
                "unknown or incompatible SVM executable magic".into(),
            ));
        };
        let load_address = u16::from_le_bytes([bytes[4], bytes[5]]);
        let entry_address = u16::from_le_bytes([bytes[6], bytes[7]]);
        let n = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        if HEADER_SIZE.checked_add(n) != Some(bytes.len()) {
            return Err(VmError::ProgramFormat("payload length mismatch".into()));
        }
        let end = (load_address as usize)
            .checked_add(n)
            .ok_or_else(|| VmError::ProgramFormat("load range overflow".into()))?;
        if end > MEMORY_SIZE {
            return Err(VmError::ProgramFormat(
                "program does not fit into 64 KiB system memory".into(),
            ));
        }
        Ok(Self {
            cpu,
            load_address,
            entry_address,
            payload: bytes[HEADER_SIZE..].to_vec(),
        })
    }
    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, VmError> {
        Self::from_bytes(&fs::read(path)?)
    }
    pub fn load_into(&self, memory: &mut Memory) -> Result<(), VmError> {
        memory.load(self.load_address, &self.payload)
    }
}
