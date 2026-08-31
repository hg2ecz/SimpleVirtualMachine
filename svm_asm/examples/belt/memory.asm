; Belt16 indirect memory example.
; Writes 0x1234 to address 0x6000 through a belt-held pointer,
; reads it back and stores the copy at 0x6002.
.load 0x0100
.entry start

.proc start
    LDI 0x6000      ; b0=address
    LDI 0x1234      ; b0=value, b1=address
    ST16 [b1],b0
    LD16 [b1]       ; b0=0x1234, b2 still contains the address
    ST16A 0x6002,b0
    HALT
.endproc
