# Belt16 instruction reference

## Control

`NOP HALT RET EI DI IRET JMP JZ JNZ JC JNC JN JNN CALL`

## Value-producing operations

`LDI imm16`, `LD8A addr`, `LD16A addr`, `LD8 [bN]`, `LD16 [bN]`, `VLD8 [bN]`, `VLD16 [bN]`, and `POP` all produce a new `b0`.

## Stores

`ST8A addr,bN`, `ST16A addr,bN`, `ST8 [bA],bV`, `ST16 [bA],bV`, `VST8 [bA],bV`, `VST16 [bA],bV`, and `PUSH bN` do not produce a belt result.

## Binary ALU

`ADD SUB AND OR XOR MUL DIV MOD SHL SHR CMP ADC SBC MULHU MULQ15 bA,bB` produce a new `b0`.

For `SUB/CMP`, `C=1` means no borrow. `ADC/SBC` use the carry chain.

## Unary ALU

`PASS NOT NEG ASR1 SHL1 SHR1 RCR1 bA` produce a new `b0`.

`SHL1` copies old bit15 to C. `SHR1` copies old bit0 to C. `RCR1` moves old C into bit15 and old bit0 into C.
