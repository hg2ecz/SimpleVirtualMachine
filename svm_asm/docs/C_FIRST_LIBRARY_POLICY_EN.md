# C-first algorithm library and the role of assembly

Canonical application algorithms live in `svm_c/lib/`. Memory, string, CRC, bit manipulation, conversion, ring-buffer and similar portable algorithms are intentionally not duplicated as nine manually synchronized ISA-specific assembly implementations.

The assembly library is primarily for MMIO/hardware primitives, target-specific ABI helpers, measured hot-path optimizations, and architecture/assembler demonstrations.

`lib/register/algorithms_demo.asm` is a small include-able example of a hand-written helper, not a parallel full standard library. If a C library routine later becomes a measured bottleneck, keep the portable C semantics as the reference and add a backend intrinsic or assembly helper only for the affected target.
