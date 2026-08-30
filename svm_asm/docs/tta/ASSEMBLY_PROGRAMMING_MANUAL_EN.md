# TTA16 assembly programming manual

On TTA16, the basic programming concept is not `ADD` or `LOAD`, but a `MOV source,destination-port` transport.

## Arithmetic

```asm
MOV R0, ALU.X
MOV R1, ALU.ADD
MOV ALU.OUT, R2
```

`ALU.X` stores the first operand. Writing `ALU.ADD` triggers addition. The result appears at the `ALU.OUT` source port.

## Memory

```asm
MOV 0x6000, MEM.ADDR
MOV 123, MEM.W16
MOV MEM.R16, R0
```

## Control

```asm
JNZ loop
CALL function
RET
```

These are assembler convenience forms for transports to `CTRL.*` ports.

See also `INSTRUCTION_REFERENCE_EN.md` and `../../../docs/TTA_ISA_SPEC_HU.md`.
