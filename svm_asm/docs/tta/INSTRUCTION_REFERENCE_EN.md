# TTA16 instruction reference

## Core

`MOV source,destination` transports one 16-bit value. If the destination is a trigger port, the transport also starts an operation.

Sources include `R0..R7`, `ALU.OUT`, memory/VRAM read ports, stack/control sources, `ZERO`, and 16-bit immediates.

Destinations include `R0..R7`, `ALU.X`, ALU trigger ports, memory/VRAM write ports, and `CTRL.*` ports.

## ALU trigger destinations

`ALU.ADD ADC SUB SBC AND OR XOR MUL MULHU MULQ15 DIV MOD SHL SHR CMP`

Unary triggers: `ALU.NOT NEG ASR1 SHL1 SHR1 RCR1`.

## Memory / VRAM

`MEM.ADDR`, `MEM.R8`, `MEM.R16`, `MEM.W8`, `MEM.W16`

`VMEM.ADDR`, `VMEM.R8`, `VMEM.R16`, `VMEM.W8`, `VMEM.W16`

## Control convenience mnemonics

`NOP HALT EI DI RET IRET PUSH POP JMP JZ JNZ JC JNC JN JNN CALL`
