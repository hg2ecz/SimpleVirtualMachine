; Four sample bytes into the single framebuffer in separate video space.
.load 0
.entry start
start:
    LDXI 0x0000
    LDAI 0x1B
    VSTA8 [X+]
    VSTA8 [X+]
    VSTA8 [X+]
    VSTA8 [X+]
    HALT
