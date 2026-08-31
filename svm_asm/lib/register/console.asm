; Register ISA console helpers. Console byte MMIO = 0xFF20.
; ABI: putc R0=char; puts R0=pointer to NUL-terminated RAM string.
; Clobbers: R1,R2; puts also advances R0.
.proc putc
    MOVI R1, 0xFF20
    STORE8 [R1], R0
    RET
.endproc

.proc newline
    MOVI R1, 0xFF20
    MOVI R0, 13
    STORE8 [R1], R0
    MOVI R0, 10
    STORE8 [R1], R0
    RET
.endproc

.proc puts
    MOVI R2, 0xFF20
puts_loop:
    LOAD8 R1, [R0+]
    CMPI R1, 0
    JZ puts_done
    STORE8 [R2], R1
    JMP puts_loop
puts_done:
    RET
.endproc
