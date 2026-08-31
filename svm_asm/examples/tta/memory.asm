; TTA16 memory ports: write/read a 16-bit word.
.entry start
.proc start
    MOV 0x6000, MEM.ADDR
    MOV 0x1234, MEM.W16
    MOV MEM.R16, R0
    HALT
.endproc
