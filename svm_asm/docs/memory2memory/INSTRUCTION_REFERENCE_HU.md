# Memory-to-Memory ISA – utasításreferencia

A Memory-to-Memory CPU általános adatregiszter helyett memóriaoperandusokon dolgozik; `A0..A3` kizárólag címregiszter. Ez a család közvetlen memory-to-memory végpontja.

8 bites műveletek: `MOV8 ADD8 SUB8 AND8 OR8 XOR8 CMP8`.

16 bites műveletek: `MOV16 ADD16 SUB16 AND16 OR16 XOR16 CMP16 MUL16 DIV16 MOD16 SHL16 SHR16 MULQ15 ADC16 SBC16 MULHU16`.

Unary/read-modify-write: `INC8 DEC8 NOT8 NEG8 INC16 DEC16 NOT16 NEG16 ASR1 RCR1 SHL1 SHR1`.

Címregiszter: `LEA`, `ADDA`, `MOVA`, `STORA`. Branch/call: rövid relatív `BRA/BZ/BNZ/BC/BNC/BN/BNN/CALLR`, illetve abszolút `JMP/JZ/JNZ/JC/JNC/JN/JNN/CALL`. VRAM: `VLD8/16`, `VST8/16`.

Az általános source descriptor immediate-et is fogadhat, ezért külön `VSTI` vagy általános immediate opcode-család nem kell. A logikai `AND/OR/XOR/NOT` teljes értékű része az ISA-nak. Hardveres floating point nincs.
