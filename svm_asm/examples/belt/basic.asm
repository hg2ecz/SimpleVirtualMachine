.load 0x0100
.entry start
start:
    LDI 10
    LDI 20
    ADD b1,b0
    ST16A 0x6000,b0
    HALT
