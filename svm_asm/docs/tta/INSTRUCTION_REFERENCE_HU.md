# TTA16 utasításreferencia

## Mag

A `MOV source,destination` egy 16 bites értéket transzportál. Trigger célport esetén a transzport egyben műveletet is indít.

Forrás lehet `R0..R7`, `ALU.OUT`, memória/VRAM olvasóport, stack/control forrás, `ZERO` vagy 16 bites immediate.

Cél lehet `R0..R7`, `ALU.X`, ALU triggerport, memória/VRAM íróport vagy `CTRL.*` port.

## ALU trigger célok

`ALU.ADD ADC SUB SBC AND OR XOR MUL MULHU MULQ15 DIV MOD SHL SHR CMP`

Unáris triggerek: `ALU.NOT NEG ASR1 SHL1 SHR1 RCR1`.

## Memória / VRAM

`MEM.ADDR`, `MEM.R8`, `MEM.R16`, `MEM.W8`, `MEM.W16`

`VMEM.ADDR`, `VMEM.R8`, `VMEM.R16`, `VMEM.W8`, `VMEM.W16`

## Control convenience mnemonikák

`NOP HALT EI DI RET IRET PUSH POP JMP JZ JNZ JC JNC JN JNN CALL`
