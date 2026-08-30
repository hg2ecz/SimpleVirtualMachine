.load 0x0100
.entry start
start:
    MOV8 [0xFF20], 72
    MOV8 [0xFF20], 105
    MOV8 [0xFF20], 10
    HALT
