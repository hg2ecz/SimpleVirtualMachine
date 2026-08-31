; Decimal output helpers for the Register ISA.
; Requires console.asm (putc). Include order is irrelevant because labels resolve globally.
.include "console.asm"

; putu16: R0=value -> decimal ASCII on console.
; Clobbers R0..R4, uses hardware stack for reversed digits.
.proc putu16
    CMPI R0, 0
    JNZ putu16_nonzero
    MOVI R0, 48
    JMP putc
putu16_nonzero:
    MOV R4, R0
    MOVI R3, 0
putu16_divloop:
    MOV R0, R4
    MOVI R1, 10
    MOD R0, R1
    PUSH R0
    MOV R0, R4
    DIV R0, R1
    MOV R4, R0
    INC R3
    CMPI R4, 0
    JNZ putu16_divloop
putu16_emit:
    POP R0
    ADDI R0, 48
    CALL putc
    DEC R3
    JNZ putu16_emit
    RET

; puti16: R0=signed value -> decimal ASCII on console.
.endproc

.proc puti16
    CMPI R0, 0
    JN puti16_negative
    JMP putu16
puti16_negative:
    PUSH R0
    MOVI R0, 45
    CALL putc
    POP R0
    NEG R0
    JMP putu16
.endproc
