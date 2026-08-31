# SVM-C general-purpose library

The general algorithm library is maintained primarily as **C source**. C is the main application language of the project, so one portable implementation can serve all nine SVM targets. Hand-written assembly is reserved mainly for low-level ABI/MMIO helpers, targeted optimization, and demonstrations.

Modules live under `svm_c/lib/` and can be included individually:

```c
include "memory.sc";
include "crc.sc";
```

or through the umbrella module:

```c
include "stdlib.sc";
```

Add the library directory with `-I svm_c/lib`. At `-O1/-O2/-Os` C-level dead-function elimination removes unreachable functions before code generation; binary output also passes through assembler procedure-GC.

## Modules

`memory.sc`: `mem_zero`, `memset`, `memcpy`, `memmove`, `memcmp`.

`string.sc`: `strlen`, `strcmp`, `strncmp`, `strcpy`, `strncpy`, `strchr`, `streq`. Strings are zero-terminated byte sequences addressed by `u16` values.

`bits.sc`: `rotl16`, `rotr16`, `popcount16`, `parity16`, `clz16`, `ctz16`, `bitreverse16`, `bswap16`.

`crc.sc`: byte checksums, CRC-8/ATM, and CRC-16/CCITT-FALSE, including incremental update routines suitable for streaming protocols.

`convert.sc`: decimal/hex parsing and `u16`/`i16` conversion to zero-terminated buffers. The initial parsing routines intentionally use wrapping 16-bit arithmetic rather than overflow reporting.

`buffer.sc`: static-memory byte ring buffer. One slot is reserved so full and empty states are distinguishable without a separate count field.

`console.sc`: in addition to the existing formatting helpers, `putstr(address)` prints a run-time generated zero-terminated buffer, `puti16` prints signed decimal, and `putbin16` prints 16 binary digits.

`stdlib.sc` includes the common integer/Q15/trigonometry, memory, string, bit, CRC, conversion, ring-buffer, software-random, and console modules. Graphics, textscreen, hardware random, and soft-float remain explicit opt-in modules.

## Assembly policy

Portable algorithms are canonical in C rather than duplicated manually for all nine ISAs. `svm_asm/lib/register/algorithms_demo.asm` is deliberately only a small demonstration of a hand-written target-specific helper.
