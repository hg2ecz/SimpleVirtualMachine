# SVM-C

SVM-C is the shared C-like compiler for all nine SimpleVirtualMachine CPU targets. It uses one frontend and one backend module per ISA.

## Targets

`register`, `stack`, `accumulator`, `memreg`, `loadstore`, `regmem`, `memory2memory`, `belt`, and `tta`.

## Build a program

```sh
cargo run --release --manifest-path svm_c/Cargo.toml -- \
  --target register -O2 program.sc program.svm
```

Emit readable assembly instead:

```sh
cargo run --release --manifest-path svm_c/Cargo.toml -- \
  --target stack -Os --emit asm program.sc program.fsasm
```

## Optimization levels

- `-O0`: no optional AST optimization; all parsed functions reach code generation, while the final assembler procedure-GC still removes unreachable emitted procedures from binary output.
- `-O1`: safe constant/algebraic simplification, strength reduction, and unreachable-function elimination.
- `-O2`: `-O1` plus local constant/copy propagation and safe dead-store elimination.
- `-Os`: size-oriented variant using the safe `-O2` transformations.

At `-O1`, `-O2`, and `-Os`, the C optimizer removes functions transitively unreachable from `main()` before static-memory layout, so they consume neither compiler-owned static RAM nor generated assembly. Independently of optimization level, final binary generation also runs the assembler procedure-GC; therefore `-O0` and `svm-c-unopt-only` may emit all parsed functions with `--emit asm`, but unreachable `.proc` blocks are still omitted from the final machine-code binary. `svm-c-unopt-only` remains free of the optional optimizer dependency.

## Language and numeric support

SVM-C supports `bool`, `i8/u8`, `i16/u16`, `i32/u32`, limited `i64/u64` storage, `int`=`i16`, and `long`=`i32`. Wide integer and `f16`/`f32` operations use address-based software-library APIs so the hardware targets remain small 16-bit CPUs.

## Documentation

The compiler documentation is indexed at [`docs/README.md`](docs/README.md). Start with [`docs/COMPILER_REFERENCE_HU.md`](docs/COMPILER_REFERENCE_HU.md) for the CLI/optimization contract and [`docs/LIBRARY_REFERENCE_HU.md`](docs/LIBRARY_REFERENCE_HU.md) for reusable libraries. It contains the language reference, numeric model, optimizer documentation, arithmetic library notes, source includes, and FFT examples.

The common machine platform and ISA specifications are deliberately kept outside the compiler under [`../docs/`](../docs/).

## Examples and libraries

Programs are under [`examples/`](examples/), including 256- and 4096-point FFT examples for Q15/u16, `f16`, and `f32`. Reusable SVM-C libraries are under [`lib/`](lib/).

The numeric regression suite is under [`examples/smoke/`](examples/smoke/) and includes full and compact all-type smoke tests.

Reusable SVM-C source libraries are under `svm_c/lib/`; `console.sc` adds newline and 16-bit numeric output helpers on top of the compiler built-ins `putc`, `puts`, and `getc`.

Graphics helper library: [`docs/GRAPHICS_LIBRARY_HU.md`](docs/GRAPHICS_LIBRARY_HU.md).

Character-screen helpers for the 40x25 framebuffer text layer are in `svm_c/lib/textscreen.sc`; see `svm_c/docs/TEXT_SCREEN_LIBRARY_HU.md`.

## C-first portable library

Portable algorithms are maintained primarily under `svm_c/lib/` as include-able SVM-C source. `stdlib.sc` collects the common memory, string, bit, CRC, conversion, ring-buffer, arithmetic/random and console helpers. Hand-written assembly is kept for low-level target-specific helpers and demonstrations rather than duplicating each portable algorithm nine times. See `docs/STANDARD_LIBRARY_HU.md` / `STANDARD_LIBRARY_EN.md`.

## Calling target-specific assembly from C

SVM-C supports target-neutral declarations such as `extern asm u16 fast(u16 x);` together with `asm_include "fast.asm";`. Put implementations under `asm/<target>/fast.asm`; the compiler emits a stable memory bridge and the final assembler procedure-GC links only reachable procedures. See `docs/ASM_INTEROP_HU.md` / `docs/ASM_INTEROP_EN.md`.
