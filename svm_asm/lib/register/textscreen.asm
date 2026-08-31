; 40x25 framebuffer text-screen helpers. Distinct from VT100 console MMIO.
; text_goto: R0=x, R1=y. text_set_colors: R0=fg, R1=bg. text_putc: R0=char.
; Clobbers R0..R3 in text_clear.
.proc text_goto
    MOVI R2, 0xFF02
    STORE8 [R2], R0
    INC R2
    STORE8 [R2], R1
    RET
.endproc

.proc text_set_colors
    MOVI R2, 0xFF04
    STORE8 [R2], R0
    INC R2
    STORE8 [R2], R1
    RET
.endproc

.proc text_home
    MOVI R0, 0
    MOVI R1, 0
    JMP text_goto
.endproc

.proc text_cr
    MOVI R2, 0xFF02
    MOVI R0, 0
    STORE8 [R2], R0
    RET
.endproc

.proc text_putc
    MOVI R2, 0xFF06
    STORE8 [R2], R0
    RET
.endproc

.proc text_clear
    MOVI R0, 0
text_clear_y:
    MOVI R2, 0xFF03
    STORE8 [R2], R0
    MOVI R1, 0
text_clear_x:
    MOVI R2, 0xFF02
    STORE8 [R2], R1
    MOVI R3, 32
    MOVI R2, 0xFF06
    STORE8 [R2], R3
    INC R1
    CMPI R1, 40
    JNZ text_clear_x
    INC R0
    CMPI R0, 25
    JNZ text_clear_y
    JMP text_home
.endproc
