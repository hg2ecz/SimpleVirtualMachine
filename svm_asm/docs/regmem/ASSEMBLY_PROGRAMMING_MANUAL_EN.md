# Register-Memory assembly programming manual

The strength of this architecture is that the second ALU operand may come directly from a register, immediate, or memory descriptor:

```asm
ADD R0, [R1+4]
AND R0, 0x7FFF
CMP R0, 10
```

Therefore separate `ANDI` or `ADDI` hardware opcode families are unnecessary; they are source-level aliases over the descriptor encoding. An ALU memory source never auto-updates its address register. Explicit RAM and VRAM load/store operations remain available.
