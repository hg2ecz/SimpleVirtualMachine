.load 0
.entry start
start:
    LDAI 72
    STA8 0xFF20
    LDAI 105
    STA8 0xFF20
    LDAI 10
    STA8 0xFF20
    HALT
