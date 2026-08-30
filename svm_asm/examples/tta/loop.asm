; Count R0 down to zero. SUB is triggered by transport to ALU.SUB.
.entry start
start:
    MOV 5, R0
loop:
    MOV R0, ALU.X
    MOV 1, ALU.SUB
    MOV ALU.OUT, R0
    JNZ loop
    HALT
