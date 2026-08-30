# Register-Memory ISA – utasításreferencia

A Register-Memory CPU 8 általános regisztert használ, de az ALU második operandusa regiszter, immediate vagy memória-descriptor is lehet. Ez a klasszikus kétcímes register-memory ellenpontja a strict Load/Store gépnek.

Fő csoportok:

- `MOV ADD SUB AND OR XOR CMP MUL DIV MOD SHL SHR MULQ15 ADC SBC MULHU Rd,src`
- `MOVI`, illetve `ADDI/SUBI/ANDI/ORI/XORI/CMPI` assembler-aliasok ugyanarra a descriptoros ALU-formára
- `NOT NEG INC DEC ASR1 SHL1 SHR1 RCR1 Rd`
- `LOAD8/16`, `STORE8/16`, `VLOAD8/16`, `VSTORE8/16`
- `JMP/JZ/JNZ/JC/JNC/JN/JNN/CALL`
- `PUSH/POP`, `ZLOAD*`, `ZSTORE*` assembler kényelmi formák

Az immediate műveletekhez nem kell külön hardveres opcode-család: ugyanaz a source descriptor kódolja a literált. ALU source auto-update szándékosan tiltott, hogy egy read-modify művelet ne módosítsa implicit módon a címregisztert.

`ADC/SBC/MULHU/RCR1` a többwordös integer és soft-float kódot segíti; FPU nincs.
