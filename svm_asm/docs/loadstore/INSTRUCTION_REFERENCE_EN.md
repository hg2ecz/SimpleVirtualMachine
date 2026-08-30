# Load/Store ISA – instruction reference

The Load/Store CPU is a 16-bit strict load/store machine with eight general registers (`R0..R7`). ALU operations use registers; system RAM and VRAM are accessed only through explicit load/store operations.

## State and flags

`Z` is zero, `N` is bit15, and `C` is carry / no-borrow after subtraction. `SHL1` writes old bit15 to `C`, `SHR1` writes old bit0, and `RCR1` rotates right through carry.

## Core instructions

- control: `NOP HALT RET EI DI IRET`
- data/ALU: `MOV CMP NOT NEG ASR1 INC DEC`
- two/three-operand ALU: `ADD SUB AND OR XOR MUL SHL SHR`
- multiword/fixed-point assists: `DIV/DIVU MOD/MODU MULQ15 ADC SBC MULHU RCR1`
- literal: `MOVI/LDI`, `ADDI`, `SUBI`, `CMPI`, `ANDI`, `ORI`, `XORI`
- memory: `LOAD8 LOAD16 STORE8 STORE16`
- VRAM: `VLOAD8 VLOAD16 VSTORE8 VSTORE16` (`VLD*`/`VST*` aliases)
- branch/call: `BRA BZ BNZ BC BNC BN BNN`, plus `JMP JZ JNZ JC JNC JN JNN CALL`
- assembler conveniences: `PUSH POP ZLOAD8/16 ZSTORE8/16`

## Immediate encoding

Small signed `ADDI/CMPI` values in `-32..31` may use the short 6-bit form. `ANDI/ORI/XORI` use small `0..63` immediates. Full 16-bit `MOVI`, `ADDI`, and `CMPI` use the long-immediate form.

`SUBI` is **not** an `ADDI -imm` alias. Long-immediate major `9`, function `3` is a dedicated `SUBI16` decode, because the numeric result could be equivalent while the observable `C` carry/no-borrow semantics are not.

## Design minimum

There is deliberately no auto-increment load/store: this target represents the clean strict load/store control point. There is no hardware floating point, 32-bit ALU, or CLZ; `f16/f32` are software.
