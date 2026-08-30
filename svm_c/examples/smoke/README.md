# SVM-C numeric smoke tests

Small, deterministic regression programs for the numeric library.  The suite
is intentionally split into layers so a broken primitive can be identified
before debugging a large workload and so code-heavy targets do not need to fit
all library code into one executable image.

Files:

- `smoke_scalar_int.sc` — bool/u8/i8/u16/i16 scalar arithmetic and signed helpers.
  Boolean checks follow the language rule `0 = false, nonzero = true`; smoke tests do not require a particular nonzero bit pattern unless an API explicitly promises `0/1`.
- `smoke_wide_int.sc` — u32/i32 arithmetic, shifts, div/mod and 32x32->64 multiplication.
- `smoke_q15.sc` — Q15 multiply/divide and cardinal-angle trigonometry.
- `smoke_f16.sc` — binary16 arithmetic, classifications and u16 conversions.
- `smoke_f32.sc` — binary32 arithmetic, classifications and u16 conversions.
- `smoke_arithmetic.sc` — integer helper library and deterministic software PRNG.
- `smoke_all.sc` — broad monolithic gate. With the current top-page-MMIO layout and unused-function elimination it fits and passes on all nine targets in the v2.3.17 reference run; the split runner remains authoritative because it localizes failures.
- `smoke_all_compact.sc` — one executable that touches every numeric storage
  family with a small representative data set and a deliberately small call
  graph.
- `run_numeric_smoke.sh` — authoritative full suite: compiles and runs every
  layer separately for all nine targets, including Belt.

The recommended full regression is:

```sh
sh svm_c/examples/smoke/run_numeric_smoke.sh target/release
```

For a quick one-image gate, use `smoke_all_compact.sc`. For a broader one-image gate, `smoke_all.sc` is also expected to fit all nine current targets. Example:

```sh
target/release/svm-c --target belt -O2 -I svm_c/lib \
  svm_c/examples/smoke/smoke_all_compact.sc /tmp/smoke_all_compact.svb
target/release/svm-rt /tmp/smoke_all_compact.svb
```

A successful program prints its smoke name and `OK`; failures print a specific
`FAIL ...` label and return non-zero.

## Static-data placement

Optimized C programs start code at `0x0100` and assemblers reject code reaching
MMIO at `0xFF00`.  Compiler statics that no longer fit in zero page therefore
live in `0xE000..0xFAFF`, not in the old `0x6000..0x6BFF` area.  The upper
region is ordinary RAM since System ROM was removed.  This prevents a large
RegMem/TTA/M2M program from growing into its own statically allocated wide
objects and later corrupting its machine code at runtime.

These tests intentionally do not test hardware entropy. `random.sc` is tested
with deterministic reference values; `hrandom.sc` is an MMIO interface and may
be backed by deterministic or physical entropy depending on the platform.

The compact gate uses individually labelled scalar checks (BOOL/U8/I8/U16/I16) so a scalar regression can be localized without running the larger suite first.
