# SVM-C documentation

This directory contains the active SVM-C language, compiler, library and regression-test documentation.

English/Hungarian document pairs must describe the same implemented language and compiler behavior. Examples and explanatory detail may differ, but supported types, syntax, ABI rules, optimization behavior, target list and limitations must not diverge.

## Language and compiler

- [`C_REFERENCE_HU.md`](C_REFERENCE_HU.md) / [`C_REFERENCE_EN.md`](C_REFERENCE_EN.md) - C subset reference.
- [`SVM_C_LANGUAGE_HU.md`](SVM_C_LANGUAGE_HU.md) / [`SVM_C_LANGUAGE_EN.md`](SVM_C_LANGUAGE_EN.md) - language overview.
- [`COMPILER_REFERENCE_HU.md`](COMPILER_REFERENCE_HU.md) - compiler CLI, targets, ABI and pipeline.
- [`OPTIMIZATION_HU.md`](OPTIMIZATION_HU.md) - optimization levels, unused-function elimination, target-aware lowering and `svm-c-unopt-only`.
- [`OPTIMIZATION.md`](OPTIMIZATION.md) - English optimization reference, semantically paired with `OPTIMIZATION_HU.md`.
- [`SOURCE_INCLUDES_HU.md`](SOURCE_INCLUDES_HU.md) - source include mechanism.
- [`ASM_INTEROP_HU.md`](ASM_INTEROP_HU.md) / [`ASM_INTEROP_EN.md`](ASM_INTEROP_EN.md) - target-neutral C declarations calling target-specific assembly modules.
- [`NUMERIC_TYPES_HU.md`](NUMERIC_TYPES_HU.md) - scalar, wide integer, Q15 and soft-float types.

## Libraries and examples

- [`LIBRARY_REFERENCE_HU.md`](LIBRARY_REFERENCE_HU.md) - complete library index.
- [`SVM_C_ARITHMETIC_LIBRARY_HU.md`](SVM_C_ARITHMETIC_LIBRARY_HU.md) - arithmetic library details.
- [`CONSOLE_LIBRARY_HU.md`](CONSOLE_LIBRARY_HU.md) - console helpers.
- [`GRAPHICS_LIBRARY_HU.md`](GRAPHICS_LIBRARY_HU.md) / [`GRAPHICS_LIBRARY_EN.md`](GRAPHICS_LIBRARY_EN.md) - 2 bpp graphics helpers.
- [`TEXT_SCREEN_LIBRARY_HU.md`](TEXT_SCREEN_LIBRARY_HU.md) - 40x25 text-screen helpers.
- [`FFT_EXAMPLES_HU.md`](FFT_EXAMPLES_HU.md) - FFT examples and reporting.
- [`SMOKE_TESTS_HU.md`](SMOKE_TESTS_HU.md) - numeric smoke/regression suite.

- [`STANDARD_LIBRARY_HU.md`](STANDARD_LIBRARY_HU.md) - általános C-first standard könyvtár (memória, string, bit, CRC, konverzió, ring buffer)
- [`STANDARD_LIBRARY_EN.md`](STANDARD_LIBRARY_EN.md) - English reference for the C-first general-purpose library
