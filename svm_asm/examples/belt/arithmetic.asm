; Belt16 arithmetic and belt ageing example.
; Result: 90 is stored at 0x6000.
.load 0x0100
.entry start

.proc start
    LDI 10          ; b0=10
    LDI 20          ; b0=20, b1=10
    ADD b1,b0       ; b0=30
    LDI 3           ; b0=3, b1=30
    MUL b1,b0       ; b0=90
    ST16A 0x6000,b0
    HALT
.endproc
