.load 0x0200
.entry start

; Copy 256 bytes from 0x3000 to 0x4000.
; R0/R1 are walking pointers, R2 is count, R3 is the byte.
.proc start
    MOVI R0, 0x3000
    MOVI R1, 0x4000
    MOVI R2, 256
copy:
    LOAD8 R3, [R0+]
    STORE8 [R1+], R3
    DEC R2
    JNZ copy
    HALT
.endproc
