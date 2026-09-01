# SVM C-Lite 1.0 release checklist

This list is intentionally short. The goal of 1.0 is not more features, but a small, understandable and reliable architecture-independent assembly-like language.

## 1. Compiler build

On a machine with a Rust toolchain:

```sh
cargo build -p svm-clite
cargo test -p svm-clite
cargo build -p svm-asm
cargo test -p svm-asm
```

Any compiler error, test failure, or `svm-clite` warning is a release blocker.

## 2. Language core

The 1.0 language set remains:

- `bool`, `i8`, `u8`, `i16`, `u16`, `void`;
- scalar variables;
- fixed-size arrays;
- one pointer level;
- `fn`, parameters and `return`;
- `if / else`;
- `while`, `break`, `continue`;
- simple arithmetic, bitwise and comparison operators;
- `load8/load16/store8/store16`;
- `vload8/vload16/vstore8/vstore16`;
- textual `include`;
- `//` and `/* ... */` comments.

No new language feature should be added before 1.0.

## 3. Simplicity gate

`svm-clite` remains:

```text
lexer -> parser -> semantic check -> CLIR -> target ASM
```

It must not gain:

- an optimizer;
- constant folding;
- SSA;
- a register allocator;
- data-flow passes;
- a linker;
- an embedded assembler;
- an SVM-C dependency;
- a general macro preprocessor.

## 4. Nine targets

The same C-Lite source must compile for all nine targets:

```text
register
stack
accumulator
memreg
loadstore
regmem
memory2memory
belt
tta
```

At minimum cover:

1. arithmetic and signed/unsigned comparisons;
2. `while`, `break`, `continue`;
3. arrays and pointers;
4. calls with several parameters;
5. 8- and 16-bit memory access;
6. volatile/MMIO;
7. `bool`;
8. globals and global arrays.

Correctness matters; performance and code size do not.

## 5. External assembler boundary

Normal compilation:

```sh
svm-clite --target register program.cl
```

must produce target assembly.

Binary output is assembled separately:

```sh
svm-asm register program.asm program.svm
```

`.proc/.endproc`, `.entry`, `.keep`, assembly includes, and unreachable-procedure filtering remain assembler responsibilities.

## 6. Include

Check:

- relative includes;
- `-I` search paths;
- include-once;
- cyclic-include errors;
- file and line reporting for missing includes.

Include stays simple textual inclusion with no macro system.

## 7. Documentation

The following must describe the actual implementation:

- `PROGRAMMING_MANUAL_HU.md` / `EN`;
- `LANGUAGE_HU.md` / `EN`;
- `CLIR_0_1_HU.md` / `EN`;
- `CODEGEN_HU.md` / `EN`;
- `DESIGN_RULES_HU.md` / `EN`;
- `ONE_ZERO_SCOPE_HU.md` / `EN`.

It is a release blocker if documentation promises a language feature unsupported by the parser or backend.

## 8. 1.0 decision

If the build is warning-free, all tests pass, all nine target smoke tests pass, and the documentation matches the implementation, the RC can be released as `1.0.0`.

The principle remains after 1.0:

> C-Lite is structured, architecture-independent assembly. Language and compiler simplicity matter more than convenience features or optimized output.


## External nine-target integration

```sh
svm_clite/scripts/test_9_targets.sh
svm_clite/scripts/report_codegen.sh svm_clite/tests/programs/array_pointer.cl
```

The script uses separate `svm-clite` and `svm-asm` executables and exercises 81 compile+assemble cases.
