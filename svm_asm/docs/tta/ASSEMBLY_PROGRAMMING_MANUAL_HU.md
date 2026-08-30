# TTA16 assembly programozási kézikönyv

A TTA16-on az alapvető programozási egység nem az `ADD` vagy `LOAD`, hanem a `MOV forrás,célport` transzport.

## Aritmetika

```asm
MOV R0, ALU.X
MOV R1, ALU.ADD
MOV ALU.OUT, R2
```

Az `ALU.X` tárolja az első operandust. Az `ALU.ADD` célport írása indítja az összeadást. Az eredmény az `ALU.OUT` forrásporton jelenik meg.

## Memória

```asm
MOV 0x6000, MEM.ADDR
MOV 123, MEM.W16
MOV MEM.R16, R0
```

## Vezérlés

```asm
JNZ loop
CALL function
RET
```

Ezek assembler convenience formák a `CTRL.*` portokra írt transzportokhoz.

Lásd még: `INSTRUCTION_REFERENCE_HU.md` és `../../../docs/TTA_ISA_SPEC_HU.md`.
