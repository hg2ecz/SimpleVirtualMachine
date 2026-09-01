# CLIR 0.1 – C-Lite's architecture-neutral assembly

CLIR is deliberately small. It is not an optimizing IR: there is no SSA, physical-register model, stack-frame model, or target-specific instruction. Virtual temporaries `%0`, `%1`, ... have no prescribed physical representation. Each target backend maps them directly to its natural machine model (for example the data stack, accumulator state, native registers, or simple static scratch storage).

Core operations are `const`, typed `load/store`, `addr`, `index`, typed `loadmem/storemem` (and volatile `v` variants), typed arithmetic/bit operations, comparisons, labels, `jz`, `jmp`, `call`, and `ret`.

Type suffixes are `.bool .u8`, `.i8`, `.u16`, `.i16`. Comparisons carry the operand type and produce a 0/1 truth value. No constant folding or other optimization is performed, so source operations remain visible in CLIR.

CLIR 0.1 is intended to remain stable through the 1.0 line. New language convenience should not require a substantially more complex IR mechanism.

## Bool

CLIR uses `.bool` for logical values. The direct backend stores them as one byte containing 0 or 1.
