# SVM-C examples

Recommended order:

1. `hello.sc` - minimal program and VT100 output
2. `language_tour.sc` - core SVM-C+ syntax
3. `video.sc` - 320x200 2-bpp VRAM and palette
4. `vt100_console.sc` - bidirectional terminal I/O
5. `optimization.sc` - compare `-O0`, `-O1`, `-O2` and `-Os`
6. `fft_q15.sc` - complete Q15 benchmark

The v1 and v2 frontends accept the same source language; v2 adds optimization levels.

- `include_demo.sc`: egyszerű saját forráskönyvtár behúzása a `lib/arithmetic.sc` fájlból.

## Arithmetic library

`arithmetic_demo.sc` demonstrates the reusable library under `svm_c/lib/`.
Compile with an include path, for example:

```sh
svm-c --target register -O2 -I svm_c/lib svm_c/examples/arithmetic_demo.sc arithmetic.svm
```

## Belt16

`belt_demo.sc` is a small target smoke example. The regular examples are also
architecture-neutral and may be compiled with `--target belt`.

```sh
svm-c --target belt -O2 svm_c/examples/belt_demo.sc belt-demo.svb
svm-c-unopt-only --target belt svm_c/examples/belt_demo.sc belt-demo-unopt.svb
```

## FFT256 / FFT4096 numeric examples

The larger FFT examples use the same radix-2 DIT structure and stage-by-stage
1/2 scaling so the integer and software-floating-point versions are directly
comparable:

| Size | u16 / Q15 | f16 soft-float | f32 soft-float |
| --- | --- | --- | --- |
| 256 | `fft256_u16.sc` | `fft256_f16.sc` | `fft256_f32.sc` |
| 4096 | `fft4096_u16.sc` | `fft4096_f16.sc` | `fft4096_f32.sc` |

`f16` is represented as an IEEE-754 binary16 bit pattern in one `u16` word.
`f32` is represented as a four-byte `u32` object and uses the address-based
soft-float API from `lib/f32.sc`. No CPU contains floating-point hardware.

The 4096-point examples intentionally do not use C static arrays. Their FFT
workspace is placed explicitly in high CPU RAM so it does not consume the
compiler's small static-object area:

- u16/Q15 and f16: real `0x8000..0x9FFF`, imag `0xA000..0xBFFF`;
- f32: real `0x8000..0xBFFF`, imag `0xC000..0xFFFF`.

The f32/4096 example therefore uses the complete upper 32 KiB RAM region for
its complex data. It does not use VRAM; VRAM remains a separate address space.
Twiddle tables are not stored in RAM. Each stage has one precomputed root and
the current twiddle is advanced by complex multiplication.

Examples for the Register target:

```sh
svm-c --target register -O2 svm_c/examples/fft256_u16.sc fft256-u16.svm
svm-c --target register -O2 -I svm_c/lib svm_c/examples/fft256_f16.sc fft256-f16.svm
svm-c --target register -O2 -I svm_c/lib svm_c/examples/fft256_f32.sc fft256-f32.svm

svm-c --target register -O2 svm_c/examples/fft4096_u16.sc fft4096-u16.svm
svm-c --target register -O2 -I svm_c/lib svm_c/examples/fft4096_f16.sc fft4096-f16.svm
svm-c --target register -O2 -I svm_c/lib svm_c/examples/fft4096_f32.sc fft4096-f32.svm
```

The same source files can be compiled for any of the nine SVM targets.

## Numeric smoke tests

Focused numeric regression programs live in `examples/smoke/`. Run
`smoke_all.sc` before large FFT benchmarks when changing the compiler, runtime,
or arithmetic libraries. See `examples/smoke/README.md` for the individual tests.

- `console_library.sc` - include-able console formatting helpers (`newline`, `putu16`, `puthex16`).

- `graphics.sc` - 2-bpp drawing helpers: pixels, lines, rectangles, circles and palette setup.
