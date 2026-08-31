# Architecture-specific assembly math demos

Portable algorithms remain canonical in `svm_c/lib`. The files under `svm_asm/lib/<target>/math.asm` are educational ISA demonstrations showing similar low-level tasks in the natural operand model of all nine VM architectures.

Every target directory now contains `math.asm`, `convert.asm`, `typed_arith.asm`, and `typed_convert.asm`, plus an `examples/<target>/typed_math_demo.asm` procedure-GC example.

The complete decimal/binary/hex string parse+format assembly reference currently remains in the Register target (`typed_convert.asm`) and in portable C (`svm_c/lib/convert.sc`). Other target `convert.asm` files are deliberately demonstration layers and should use the C implementation as the behavioural reference when extended.

Full IEEE-754 f16/f32 software arithmetic remains canonical in C (`f16.sc`, `f32.sc`); the assembly demos focus on representation helpers and integer primitives rather than maintaining nine independent soft-float implementations.
