# Memory-to-Memory ISA – instruction reference

The CPU operates directly on memory operands; `A0..A3` are address registers only, not general data registers. This is the direct memory-to-memory endpoint of the architecture family.

8-bit operations: `MOV8 ADD8 SUB8 AND8 OR8 XOR8 CMP8`.

16-bit operations: `MOV16 ADD16 SUB16 AND16 OR16 XOR16 CMP16 MUL16 DIV16 MOD16 SHL16 SHR16 MULQ15 ADC16 SBC16 MULHU16`.

Unary/read-modify-write operations: `INC8 DEC8 NOT8 NEG8 INC16 DEC16 NOT16 NEG16 ASR1 RCR1 SHL1 SHR1`.

Address-register operations: `LEA`, `ADDA`, `MOVA`, `STORA`. Branch/call forms are short relative `BRA/BZ/BNZ/BC/BNC/BN/BNN/CALLR` and absolute `JMP/JZ/JNZ/JC/JNC/JN/JNN/CALL`. VRAM uses `VLD8/16` and `VST8/16`.

The general source descriptor can encode immediates, so separate `VSTI` or general immediate opcode families are unnecessary. `AND/OR/XOR/NOT` are full members of the ISA. No hardware floating point is provided.
