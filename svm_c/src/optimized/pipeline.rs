use crate::model::*;
use crate::{
    backend_accumulator, backend_belt, backend_loadstore, backend_memory2memory, backend_memreg,
    backend_register, backend_regmem, backend_stack, backend_tta,
};

pub fn compile_source(source: &str, target: Target, opt: OptLevel) -> Result<String, String> {
    let program = crate::frontend::parse_source(source)?;
    // Validate source semantics before optimization so an optimizer cannot hide
    // an invalid call/variable inside a constant-dead branch.
    crate::semantic::validate_semantics(&program)?;
    // At optimized levels prune unreachable functions before static allocation.
    // Otherwise dead library functions still consume the scarce zero-page/high-static
    // area even though their code is removed later.  This also reduces the chance
    // that large umbrella includes force data into the 0xE000 region unnecessarily.
    let mut program = program;
    if opt != OptLevel::O0 {
        crate::optimized::optimizer::eliminate_unreachable_functions(&mut program);
    }
    // Allocate before lowering fixed arrays so their base addresses are stable.
    let layout = crate::layout::make_layout(&program, target)?;
    let program = crate::layout::lower_subset_plus(program, &layout)?;
    let program = crate::optimized::optimizer::optimize_program(program, opt);
    crate::semantic::validate(&program)?;
    match target {
        Target::Register => backend_register::emit(&program, &layout, opt),
        Target::Stack => backend_stack::emit(&program, &layout, opt),
        Target::Accumulator => backend_accumulator::emit(&program, &layout, opt),
        Target::MemReg => backend_memreg::emit(&program, &layout, opt),
        Target::LoadStore => backend_loadstore::emit(&program, &layout, opt),
        Target::RegMem => backend_regmem::emit(&program, &layout, opt),
        Target::Memory2Memory => backend_memory2memory::emit(&program, &layout, opt),
        Target::Belt => backend_belt::emit(&program, &layout, opt),
        Target::Tta => backend_tta::emit(&program, &layout, opt),
    }
}
