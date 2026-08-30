use crate::model::*;
use crate::{
    backend_accumulator, backend_belt, backend_loadstore, backend_memory2memory, backend_memreg,
    backend_register, backend_regmem, backend_stack, backend_tta,
};

/// Compile source without any AST optimization pass.
///
/// This deliberately preserves the source-level expression/statement structure
/// after mandatory language lowering. Backend instruction selection is shared
/// with the normal compiler so comparisons isolate the optimizer itself.
pub fn compile_source_unoptimized(source: &str, target: Target) -> Result<String, String> {
    let program = crate::frontend::parse_source(source)?;
    crate::semantic::validate_semantics(&program)?;
    let layout = crate::layout::make_layout(&program, target)?;
    let program = crate::layout::lower_subset_plus(program, &layout)?;
    crate::semantic::validate(&program)?;

    match target {
        Target::Register => backend_register::emit(&program, &layout, OptLevel::O0),
        Target::Stack => backend_stack::emit(&program, &layout, OptLevel::O0),
        Target::Accumulator => backend_accumulator::emit(&program, &layout, OptLevel::O0),
        Target::MemReg => backend_memreg::emit(&program, &layout, OptLevel::O0),
        Target::LoadStore => backend_loadstore::emit(&program, &layout, OptLevel::O0),
        Target::RegMem => backend_regmem::emit(&program, &layout, OptLevel::O0),
        Target::Memory2Memory => backend_memory2memory::emit(&program, &layout, OptLevel::O0),
        Target::Belt => backend_belt::emit(&program, &layout, OptLevel::O0),
        Target::Tta => backend_tta::emit(&program, &layout, OptLevel::O0),
    }
}
