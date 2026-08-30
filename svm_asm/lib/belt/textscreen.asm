; 40x25 framebuffer text-screen helpers.
; text_goto: b1=x,b0=y. text_set_colors: b1=fg,b0=bg. text_putc: b0=char.
; Scratch words: 0x00F2 (y), 0x00F4 (x).
text_goto:
    ST8A 0xFF03,b0
    ST8A 0xFF02,b1
    RET
text_set_colors:
    ST8A 0xFF05,b0
    ST8A 0xFF04,b1
    RET
text_home:
    LDI 0
    ST8A 0xFF02,b0
    ST8A 0xFF03,b0
    RET
text_cr:
    LDI 0
    ST8A 0xFF02,b0
    RET
text_putc:
    ST8A 0xFF06,b0
    RET
text_clear:
    LDI 0
    ZST16 0xF2,b0
text_clear_y:
    ZLD16 0xF2
    ST8A 0xFF03,b0
    LDI 0
    ZST16 0xF4,b0
text_clear_x:
    ZLD16 0xF4
    ST8A 0xFF02,b0
    LDI 32
    ST8A 0xFF06,b0
    ZLD16 0xF4
    LDI 1
    ADD b1,b0
    ZST16 0xF4,b0
    ZLD16 0xF4
    LDI 40
    CMP b1,b0
    JNZ text_clear_x
    ZLD16 0xF2
    LDI 1
    ADD b1,b0
    ZST16 0xF2,b0
    ZLD16 0xF2
    LDI 25
    CMP b1,b0
    JNZ text_clear_y
    JMP text_home
