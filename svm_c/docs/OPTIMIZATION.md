# SVM-C optimization reference

## Goal

SVM-C uses small, auditable optimization passes. The goal is useful code-size and execution-cost improvement without a large SSA/CFG-based optimizer.

Mandatory lowering is not considered optimization: transformations required to map the common AST naturally to one of the nine backends are part of ordinary code generation.

## Optimization levels

- `-O0` — no optional AST optimization; useful for direct source-to-assembly inspection.
- `-O1` — safe local simplification and constant folding.
- `-O2` — `-O1` plus local constant/copy propagation, simple dead-store elimination, and target-aware local improvements.
- `-Os` — size-oriented optimized pipeline, preferring the smallest available encodings.
- `svm-c-unopt-only` — separate reference entry point that does not declare or call the optional optimizer module and accepts no `-O` option.

## Unused function elimination

At `-O1`, `-O2`, and `-Os`, the compiler traverses the direct call graph transitively from `main()` and retains only reachable functions.

The pass runs **before static-memory layout**, so dead functions:

- generate no machine code;
- consume no RAM for their local/compiler-owned static objects;
- do not make a large included library occupy code/data space merely because it was included.

`-O0` and `svm-c-unopt-only` deliberately retain every parsed function for educational and code-generation comparison.

SVM-C currently has no function pointers, so direct-call reachability is complete for the supported language model.

## Shared AST optimizations

Typical examples:

- constant folding (`2 + 3 -> 5`);
- neutral-element elimination (`x + 0`, `x * 1`);
- recognition of native `INC/DEC` forms;
- safe power-of-two strength reduction;
- constant comparison and constant control-flow simplification;
- removal of `if (0)` / `while (0)`;
- `-O2` local constant and copy propagation;
- removal of directly overwritten, side-effect-free stores.

Dead-store elimination may delete an earlier assignment only when the next right-hand side does **not** read that same variable. Therefore this must remain intact:

```c
hi = ah + bh;
hi = hi + carry;
```

Function calls and control-flow joins conservatively invalidate local value knowledge. The compiler does not perform full alias analysis or global data-flow analysis.

## Loop-condition safety

`-O2` and `-Os` do not freeze a value known before a loop into a repeatedly evaluated `while` or `for` condition. Runtime loads of induction variables remain where required.

## Backend-level instruction selection

Not every shortening is an optimizer pass. If an ISA provides a native direct form for an AST operation, selecting it is mandatory lowering/instruction selection and may therefore happen even at `-O0`.

Examples:

- Register: `ADDI`, native `SUBI`, `CMPI`, compact GPR forms, `INC/DEC/SHL1/SHR1`;
- MemReg: `ADDI/SUBI/ANDI/ORI/XORI/CMPI` and hot expression scratch at `0x000E..0x000F`;
- Stack: short literals, one-byte ALU/stack primitives, lowering that naturally benefits from the TOS/NOS cache;
- linear memory traversal: the natural post-increment/pre-decrement forms of the target ISA.

For literal-string `puts()`, the Register and MemReg backends load the string address once before the output loop rather than once per character.

## MemReg hot scratch

For the MemReg target, the compiler reserves the 16-bit pair `0x000E..0x000F` as expression scratch. This lies in the hot direct-file window, allowing frequent `MOV16`, `ADD`, and `AND` operations to use shorter encodings.

`0x0000..0x000D` remains user static space; this scratch reservation does not alter other targets' layouts.

## `svm-c-unopt-only`

The reference compiler uses the same frontend, semantics, ABI, and backends as normal `svm-c`, but omits the optional optimizer layer.

Examples:

```bash
svm-c-unopt-only --target register program.sc program.svm
svm-c-unopt-only --target stack -I lib --emit asm program.sc program.asm
```

`-O0/-O1/-O2/-Os` are deliberately errors for this binary.

## Source-structure boundary

```text
common/      lexer/parser/AST/semantic/layout
backend/     code generation for the nine ISAs
optimized/   optional optimizer + optimized pipeline
unopt/       optimizer-free pipeline
```

Language rules and static layout are shared. Instruction selection and ISA-specific ABI details belong to the backends.

## Deliberately excluded optimizations

The compiler currently has no:

- SSA;
- global liveness or global register allocation;
- aggressive inlining;
- loop unrolling;
- complex alias analysis;
- large interprocedural optimizer.

Such features are justified only if real programs show a benefit that clearly outweighs the additional compiler complexity.

## Related documentation

- [Compiler reference](COMPILER_REFERENCE_HU.md)
- [C language reference](C_REFERENCE_EN.md)
- [Numeric types](NUMERIC_TYPES_HU.md)
- [Smoke tests](SMOKE_TESTS_HU.md)
