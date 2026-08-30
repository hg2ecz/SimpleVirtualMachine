.load 0x0100
.entry start
start:
    LDI R1, 0xFF20
    LDI R0, 72
    ST8 [R1], R0
    LDI R0, 105
    ST8 [R1], R0
    LDI R0, 10
    ST8 [R1], R0
    HALT
