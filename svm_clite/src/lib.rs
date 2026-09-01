mod ast;
mod backend;
mod ir;
mod lexer;
mod parser;
mod semantic;
mod target;

use ast::Program;
pub use target::Target;

fn frontend(source: &str) -> Result<Program, String> {
    let program = parser::parse(source)?;
    semantic::validate(&program)?;
    Ok(program)
}

/// Emit the small target-neutral C-Lite IR.
pub fn compile_to_ir(source: &str) -> Result<String, String> {
    ir::emit_ir(&frontend(source)?)
}

/// Compile directly from C-Lite through CLIR to target assembly.
/// No SVM-C layer and no optimizer are used.
pub fn compile_to_asm(source: &str, target: Target) -> Result<String, String> {
    let clir = compile_to_ir(source)?;
    backend::compile(&clir, target)
}

/// Parse and validate only. C-Lite deliberately has no optimizer.
pub fn check(source: &str) -> Result<(), String> {
    frontend(source).map(|_| ())
}

#[cfg(test)]
mod backend_smoke_tests {
    use super::*;

    fn targets() -> [Target; 9] {
        [
            Target::Register,
            Target::Stack,
            Target::Accumulator,
            Target::MemReg,
            Target::LoadStore,
            Target::RegMem,
            Target::Memory2Memory,
            Target::Belt,
            Target::Tta,
        ]
    }

    fn assert_all_targets(source: &str) {
        for target in targets() {
            let asm =
                compile_to_asm(source, target).expect("program should lower for every target");
            assert!(!asm.trim().is_empty());
            assert!(asm.contains(".proc"));
        }
    }

    #[test]
    fn arithmetic_and_signed_compare_reach_all_targets() {
        assert_all_targets(
            "fn main()->u16{i16 a=-3;i16 b=5;u16 x=7;u16 y=2;if(a<b){x=x*y;}return x/y;}",
        );
    }

    #[test]
    fn while_reaches_all_targets() {
        assert_all_targets("fn main()->u16{u16 i=0;u16 s=0;while(i<10){s=s+i;i=i+1;}return s;}");
    }

    #[test]
    fn array_and_pointer_reach_all_targets() {
        assert_all_targets(
            "fn sum(u16* p,u16 n)->u16{u16 i=0;u16 s=0;while(i<n){s=s+p[i];i=i+1;}return s;} fn main()->u16{u16 a[4];a[0]=1;a[1]=2;a[2]=3;a[3]=4;return sum(&a[0],4);}",
        );
    }

    #[test]
    fn five_parameters_reach_all_targets() {
        assert_all_targets(
            "fn add5(u16 a,u16 b,u16 c,u16 d,u16 e)->u16{return a+b+c+d+e;} fn main()->u16{return add5(1,2,3,4,5);}",
        );
    }

    #[test]
    fn raw_memory_and_mmio_reach_all_targets() {
        assert_all_targets(concat!(
            "fn main()->u16{",
            "store8(0x1000,65);store16(0x1002,0x1234);",
            "vstore8(0xff00,66);vstore16(0xff02,0xabcd);",
            "load8(0x1000);vload8(0xff00);",
            "return load16(0x1002)+vload16(0xff02);",
            "}"
        ));
    }

    #[test]
    fn break_and_continue_reach_all_targets() {
        assert_all_targets(
            "fn main()->u16{u16 i=0;u16 s=0;while(i<10){i=i+1;if(i==3){continue;}if(i==8){break;}s=s+i;}return s;}",
        );
    }

    #[test]
    fn global_scalar_and_array_reach_all_targets() {
        assert_all_targets(
            "u16 counter=2;u8 data[4];fn main()->u16{data[0]=1;data[1]=2;return counter+data[0]+data[1];}",
        );
    }

    #[test]
    fn bool_reaches_all_targets() {
        assert_all_targets(
            "fn less(u16 a,u16 b)->bool{return a<b;} fn main()->u16{bool ready=less(1,2);if(ready){return 1;}return 0;}",
        );
    }

    #[test]
    fn byte_parameter_call_reaches_all_targets() {
        assert_all_targets("fn echo(u8 x)->u8{return x;} fn main()->u16{echo(7);return 0;}");
    }
}
