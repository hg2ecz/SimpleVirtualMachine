.include "typed_arith.asm"
.include "typed_convert.asm"
.include "console.asm"
.entry start

.proc start
    ; 1234 + 4321, then print result as decimal using the existing console helper.
    MOVI R0,1234
    MOVI R1,4321
    CALL u16_add
    CALL putu16

    ; Demonstrate typed 8-bit wraparound: 250 + 20 = 14.
    MOVI R0,250
    MOVI R1,20
    CALL u8_add
    CALL putu16
    HALT
.endproc
