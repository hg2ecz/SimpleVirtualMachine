\ Decimal output helpers for the Stack ISA.
.include "console.asm"

\ putu16 ( u -- ) recursive decimal conversion.
.proc putu16
    DUP 10 ULT IF
        48 ADD putc
        RET
    THEN
    DUP 10 DIV CALL putu16
    10 MOD 48 ADD putc
    RET
.endproc

\ puti16 ( n -- ) signed decimal conversion.
.proc puti16
    DUP 0< IF 45 putc NEG THEN
    putu16
    RET
.endproc
