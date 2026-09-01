# SVM C-Lite 1.0 simplicity boundary

C-Lite is not intended to be C-compatible. It is a small structured architecture-neutral assembly-like language that hides the syntax of the nine SVM ISAs.

Version 1.0 contains only i8/u8/i16/u16/void, one pointer level, fixed arrays, globals/locals, functions, if/else, while, break/continue, basic arithmetic/bit/comparison operators, address/dereference/indexing, raw 8/16-bit memory and volatile MMIO operations, textual include-once, comments, and diagnostic check/IR/ASM output.

There is deliberately no optimizer, constant folding, for/switch/goto, structs/unions/enums/typedefs, macros, compound assignment syntax, multiple pointer levels, dynamic allocation, recursion, SSA, register allocation, linker, or machine-code encoder.

The pipeline is C-Lite -> validation -> CLIR 0.1 -> direct target assembly -> external svm-asm. Unused `.proc` removal belongs to the assembler, not the compiler.

## Boolean representation

`bool` is one byte in memory, always 0 or 1. There is no bit packing. Comparisons produce `bool`.
