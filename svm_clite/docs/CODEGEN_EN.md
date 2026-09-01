# C-Lite code generation

The C-Lite compiler is intentionally simple. It does not use another C compiler, does not embed the assembler, and does not optimize.

The compilation pipeline is:

```text
C-Lite source
    ↓
lexer
    ↓
parser
    ↓
semantic validation
    ↓
CLIR 0.1
    ├→ Register ASM
    ├→ Stack ASM
    ├→ Accumulator ASM
    ├→ MemReg ASM
    ├→ LoadStore ASM
    ├→ RegMem ASM
    ├→ Memory-to-Memory ASM
    ├→ Belt ASM
    └→ TTA ASM
```

CLIR is the only architecture-neutral assembly layer. A second register-shaped pseudo-machine should not be imposed on every ISA.

## Stack backend

The Stack backend lowers CLIR directly to Stack16 assembly. `%temp` values live on the VM data stack. For example:

```text
load.u16 %0, a
load.u16 %1, b
mul.u16 %2, %0, %1
const.u16 %3, 3
add.u16 %4, %2, %3
ret %4
```

becomes code of this form:

```asm
0x8000 @
0x8002 @
MUL
3
ADD
RET
```

There is no R0..R7 emulation and no static RAM slot for every CLIR temporary. Real C-Lite locals and globals may still use static memory because recursion is intentionally forbidden.

## Nine target-owned backends

All nine targets lower CLIR directly to their own assembly. There is no canonical or other shared CPU model. Only target-neutral CLIR data layout may be shared: static addresses for variables, parameters, and temporaries when a target needs them. Stack uses the VM data stack, Accumulator uses A/X, MemReg uses W/file registers, Memory2Memory uses memory operands, Belt uses belt values, and TTA emits transports directly.

## No optimization

There is no constant folding, SSA, register allocation, dead-code elimination, inlining, or instruction scheduling. Natural target lowering is not an optimization pass.

## The assembler is a separate program

`svm-clite` writes target assembly. A separate `svm-asm` creates the binary. `.proc/.endproc`, assembly includes, and removal of unused procedures belong to the assembler.
