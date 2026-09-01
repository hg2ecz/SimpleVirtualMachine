mod accumulator;
mod belt;
mod layout;
mod loadstore;
mod memory2memory;
mod memreg;
mod register;
mod regmem;
mod stack;
mod tta;

use crate::target::Target;

pub fn compile(clir: &str, target: Target) -> Result<String, String> {
    match target {
        Target::Stack => stack::lower(clir),
        Target::Register => register::lower(clir),
        Target::Accumulator => accumulator::lower(clir),
        Target::MemReg => memreg::lower(clir),
        Target::LoadStore => loadstore::lower(clir),
        Target::RegMem => regmem::lower(clir),
        Target::Memory2Memory => memory2memory::lower(clir),
        Target::Belt => belt::lower(clir),
        Target::Tta => tta::lower(clir),
    }
}
