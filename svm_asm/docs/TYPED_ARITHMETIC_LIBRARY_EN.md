# Typed assembly arithmetic reference library

Portable algorithms remain canonical in SVM-C. Assembly is intentionally a reference/demo layer for ABI, multiword arithmetic and hand optimization. The complete assembly sample lives under the Register ISA as `typed_arith.asm` and `typed_convert.asm`.

It demonstrates typed `u8/i8/u16/i16` arithmetic, `u32/i32` add/sub/multiply primitives, IEEE binary16 representation helpers, exact 16x16->32 multiplication, and NUL-terminated decimal/hex/binary integer conversion. `f16` and `f32` full software IEEE arithmetic remains canonical in `svm_c/lib/f16.sc` and `f32.sc`; maintaining nine hand-written soft-float copies would conflict with the C-first library policy.

Procedure GC removes unused routines from a full include.
