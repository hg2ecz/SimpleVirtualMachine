# SimpleVirtualMachine

SimpleVirtualMachine is a common 16-bit virtual hardware platform used to compare nine different CPU architecture styles under the same memory, video, MMIO, runtime, assembler, and C-like compiler environment.

## Repository layout

```text
svm_asm/       assembler with nine ISA backends and assembly documentation
svm_rt/        common runtime and the nine CPU cores
svm_c/         SVM-C compiler, numeric libraries, examples, and C documentation
docs/          common platform and ISA documentation
```

The nine targets are `register`, `stack`, `accumulator`, `memreg`, `loadstore`, `regmem`, `memory2memory`, `belt`, and `tta`.

## Common platform

The CPU address space is 64 KiB. `0x0000..0xFEFF` is one contiguous RAM region and `0xFF00..0xFFFF` is the MMIO page. Video memory is a separate 16 KiB address space; the 320x200x2-bpp framebuffer therefore does not consume CPU RAM. There is no CPU-visible system ROM.

See [`docs/README.md`](docs/README.md) for the platform and ISA documentation index.

## Build

```sh
cargo build --workspace --release
```

For build notes, see [`docs/BUILD_HU.md`](docs/BUILD_HU.md).

## Assembler

`svm_asm` contains the assembler, architecture-specific assembly manuals, instruction references, handwritten assembly examples, target-specific include libraries, and a common procedure-GC pass. Reusable routines use `.proc/.endproc`; complete libraries may be included while unreachable procedures are omitted from the final binary.

See [`svm_asm/README.md`](svm_asm/README.md) and [`svm_asm/docs/README.md`](svm_asm/docs/README.md).

## SVM-C compiler

`svm_c` contains a shared frontend and nine ISA backends. The main compiler supports `-O0`, `-O1`, `-O2`, and `-Os`; `svm-c-unopt-only` intentionally omits the optimizer for educational/reference comparisons.

At `-O1`, `-O2`, and `-Os`, functions that are not transitively reachable from `main()` are removed before static memory layout, so they consume neither generated assembly nor compiler-owned static RAM. At `-O0` and in `svm-c-unopt-only`, all parsed functions still reach C-level code generation, but final binary generation always runs assembler procedure-GC, so unreachable emitted `.proc` blocks do not consume machine-code space. `--emit asm` preserves those blocks for inspection.

See [`svm_c/README.md`](svm_c/README.md) and [`svm_c/docs/README.md`](svm_c/docs/README.md).

## Numeric support

The CPU cores remain 16-bit integer designs. Multiword integer, Q15, `f16`, and `f32` arithmetic is implemented in software libraries, with small hardware assists such as carry-aware arithmetic and high-half multiplication where they provide good cost/performance value. Floating point is software-only.

The numeric smoke suite includes both a full and a compact cross-target test. In the v2.3.17 baseline, both pass on all nine architectures.

## Documentation organization

- [`docs/`](docs/) - common platform, MMIO/video reference, ISA specifications, ISA design reviews, and implementation status.
- [`svm_asm/docs/`](svm_asm/docs/) - assembler usage, assembly programming manuals, instruction references, includes, console/graphics/text helpers.
- [`svm_rt/docs/`](svm_rt/docs/) - runtime usage, executable format, VM structure, host I/O behavior, and cycle model.
- [`svm_c/docs/`](svm_c/docs/) - SVM-C language, compiler/optimizer contract, ABI/numeric libraries, smoke tests, graphics/text/console helpers, and FFT examples.
