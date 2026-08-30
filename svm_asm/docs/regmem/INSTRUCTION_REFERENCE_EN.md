# Register-Memory ISA – instruction reference

The Register-Memory CPU has eight general registers, but the second ALU operand may be a register, immediate, or memory descriptor. It is the classic two-address register-memory counterpart to the strict Load/Store target.

Main groups:

- `MOV ADD SUB AND OR XOR CMP MUL DIV MOD SHL SHR MULQ15 ADC SBC MULHU Rd,src`
- `MOVI`, plus `ADDI/SUBI/ANDI/ORI/XORI/CMPI` assembler aliases over the same descriptor-based ALU form
- `NOT NEG INC DEC ASR1 SHL1 SHR1 RCR1 Rd`
- `LOAD8/16`, `STORE8/16`, `VLOAD8/16`, `VSTORE8/16`
- `JMP/JZ/JNZ/JC/JNC/JN/JNN/CALL`
- `PUSH/POP`, `ZLOAD*`, `ZSTORE*` assembler conveniences

Immediate operations do not need a separate hardware opcode family: the same source descriptor encodes the literal. ALU source auto-update is deliberately forbidden so a read/modify operation cannot implicitly alter the address register.

`ADC/SBC/MULHU/RCR1` assist multiword integer and soft-float code. There is no hardware FPU.
