.load 0x0100
.entry start
start:
    MOVI R1, 0xFF20
    MOVI R0, 72
    STORE8 [R1], R0
    MOVI R0, 105
    STORE8 [R1], R0
    MOVI R0, 10
    STORE8 [R1], R0
    HALT
