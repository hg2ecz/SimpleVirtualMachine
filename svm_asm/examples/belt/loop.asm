; Belt16 counted loop using arithmetic flags.
; Counts 10 down to 0 in RAM at 0x6000.
.load 0x0100
.entry start

start:
    LDI 10
    ST16A 0x6000,b0

loop:
    LD16A 0x6000
    LDI 1
    SUB b1,b0       ; C=no-borrow, Z/N from the result
    ST16A 0x6000,b0 ; store does not change flags
    JNZ loop

    HALT
