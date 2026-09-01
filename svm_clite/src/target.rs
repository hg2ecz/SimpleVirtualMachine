#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Register,
    Stack,
    Accumulator,
    MemReg,
    LoadStore,
    RegMem,
    Memory2Memory,
    Belt,
    Tta,
}

impl Target {
    pub fn parse(name: &str) -> Result<Self, String> {
        Ok(match name {
            "register" | "reg" => Self::Register,
            "stack" => Self::Stack,
            "accumulator" | "acc" => Self::Accumulator,
            "memreg" => Self::MemReg,
            "loadstore" => Self::LoadStore,
            "regmem" => Self::RegMem,
            "memory2memory" | "m2m" => Self::Memory2Memory,
            "belt" => Self::Belt,
            "tta" => Self::Tta,
            _ => return Err(format!("unknown target {name}")),
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Register => "register",
            Self::Stack => "stack",
            Self::Accumulator => "accumulator",
            Self::MemReg => "memreg",
            Self::LoadStore => "loadstore",
            Self::RegMem => "regmem",
            Self::Memory2Memory => "memory2memory",
            Self::Belt => "belt",
            Self::Tta => "tta",
        }
    }
}
