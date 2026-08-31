; CALL/RET are assembler conveniences for transports to/from control ports.
.entry start
.proc start
    MOV 7, R0
    CALL twice
    HALT

.endproc

.proc twice
    MOV R0, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R0
    RET
.endproc
