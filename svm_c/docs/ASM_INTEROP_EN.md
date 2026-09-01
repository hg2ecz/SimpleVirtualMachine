# SVM-C / assembly interoperability

SVM-C can call a target-specific assembly module from target-neutral C source.

```c
asm_include "crc_fast.asm";
extern asm u16 crc16_fast(u16 addr, u16 len);
```

Recommended project layout:

```text
program.sc
asm/register/crc_fast.asm
asm/stack/crc_fast.asm
asm/accumulator/crc_fast.asm
...
asm/tta/crc_fast.asm
```

For the selected target, SVM-C searches the source directory as a direct-file fallback, then `asm/<target>/`, then `-I DIR/<target>/`, then `-I DIR`, and finally the built-in `svm_asm/lib/<target>/` directory. Multi-target projects should prefer `asm/<target>/`; the direct fallback is mainly useful for local or single-target modules.

For every `extern asm` function the compiler emits a small target-native C wrapper. The assembly implementation is named `__asm_<C-name>`. Parameters and results cross the C/ASM boundary through stable memory symbols:

```text
__cabi_<function>_<parameter>
__cabi_<function>_return
```

The implementation reads its parameters from these locations and writes a non-void result to the return location before `RET`. Bridge slots use the declared C type width: `i8/u8/bool` use one byte and `i16/u16` use two bytes. Wide (`i32/u32/i64/u64`) values are not passed or returned directly by the current language; pass a `u16` address to storage instead. Multi-byte data follows the VM's normal little-endian memory representation. This bridge deliberately hides the differences between register, stack, accumulator, belt and other native call ABIs.

`--emit asm` preserves the logical `.include`, wrapper and bridge symbols. Binary emission resolves assembly includes, expands `.equ` constants and then runs procedure GC, so unused procedures in an included module are not linked into the program.

See `svm_c/examples/extern_asm.sc` and `svm_c/examples/asm/<target>/interop_demo.asm`.

## Reserved names

The `__asm_` and `__cabi_` prefixes are reserved for the compiler's C/ASM bridge. C globals and functions may not start with either prefix. Assembly implementations use the generated `__asm_<C-name>` entry point and `__cabi_*` symbols, but user-defined unrelated symbols should not use these prefixes.

## Parameter types

`extern asm` follows the same scalar parameter and return-type restrictions as ordinary SVM-C functions. The bridge allocates each parameter using its C type width; the ASM routine must use the corresponding 8- or 16-bit access. Wide (`i32/u32/i64/u64`) values should continue to be passed by address. A `void` function has no `__cabi_<function>_return` symbol.
