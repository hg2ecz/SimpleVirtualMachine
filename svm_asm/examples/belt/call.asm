; Belt16 CALL/RET example.
; square() consumes the current b0 value and leaves its result in new b0.
; Result: 49 at 0x6000.
.load 0x0100
.entry start

start:
    LDI 7
    CALL square
    ST16A 0x6000,b0
    HALT

square:
    MUL b0,b0
    RET
