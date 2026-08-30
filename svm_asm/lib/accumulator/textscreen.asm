; 40x25 framebuffer text-screen helpers.
; text_goto: X=x, Y=y. text_set_colors: X=fg, Y=bg. text_putc: A=char.
text_goto:
    TXA
    STA8 0xFF02
    TYA
    STA8 0xFF03
    RET
text_set_colors:
    TXA
    STA8 0xFF04
    TYA
    STA8 0xFF05
    RET
text_home:
    LDXI 0
    LDYI 0
    JMP text_goto
text_cr:
    LDAI 0
    STA8 0xFF02
    RET
text_putc:
    STA8 0xFF06
    RET
text_clear:
    LDAI 0
    STA8 0x00F2
text_clear_y:
    LDA8 0x00F2
    STA8 0xFF03
    LDAI 0
    STA8 0x00F3
text_clear_x:
    LDA8 0x00F3
    STA8 0xFF02
    LDAI 32
    STA8 0xFF06
    LDA8 0x00F3
    INC
    STA8 0x00F3
    CMPI 40
    JNZ text_clear_x
    LDA8 0x00F2
    INC
    STA8 0x00F2
    CMPI 25
    JNZ text_clear_y
    JMP text_home
