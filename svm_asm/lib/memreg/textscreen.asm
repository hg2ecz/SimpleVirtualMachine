; 40x25 framebuffer text-screen helpers.
; text_goto: FSR0=x, FSR1=y. text_set_colors: FSR0=fg, FSR1=bg. text_putc: W=char.
.proc text_goto
    F02W
    FSR0I 0xFF02
    STB0
    F12W
    FSR0I 0xFF03
    STB0
    RET
.endproc

.proc text_set_colors
    F02W
    FSR0I 0xFF04
    STB0
    F12W
    FSR0I 0xFF05
    STB0
    RET
.endproc

.proc text_home
    LDI 0
    FSR0I 0xFF02
    STB0
    FSR0I 0xFF03
    STB0
    RET
.endproc

.proc text_cr
    LDI 0
    FSR0I 0xFF02
    STB0
    RET
.endproc

.proc text_putc
    FSR0I 0xFF06
    STB0
    RET
.endproc

.proc text_clear
    LDI 0
    MOV8 W,0xF2
text_clear_y:
    MOV8 0xF2,W
    FSR0I 0xFF03
    STB0
    LDI 0
    MOV8 W,0xF3
text_clear_x:
    MOV8 0xF3,W
    FSR0I 0xFF02
    STB0
    LDI 32
    FSR0I 0xFF06
    STB0
    INC 0xF3
    MOV8 0xF3,W
    CMPI 40
    JNZ text_clear_x
    INC 0xF2
    MOV8 0xF2,W
    CMPI 25
    JNZ text_clear_y
    JMP text_home
.endproc
