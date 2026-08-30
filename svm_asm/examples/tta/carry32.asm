; 0x0000_FFFF + 1 using ADD then ADC.
.entry start
start:
    MOV 0xFFFF, R0       ; low
    MOV 0x0000, R1       ; high
    MOV R0, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R0
    MOV R1, ALU.X
    MOV 0, ALU.ADC
    MOV ALU.OUT, R1
    HALT
