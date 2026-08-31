; TTA16: 10 + 20 = 30, result in R2.
.entry start
.proc start
    MOV 10, R0
    MOV R0, ALU.X
    MOV 20, ALU.ADD
    MOV ALU.OUT, R2
    HALT
.endproc
