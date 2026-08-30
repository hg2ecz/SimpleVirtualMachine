; 40x25 framebuffer text-screen helpers.
; text_goto ( x y -- ); text_set_colors ( fg bg -- ); text_putc ( ch -- ).
; Scratch: 0x00F2..0x00F3.
text_goto:
    0xFF03 STORE8
    0xFF02 STORE8
    RET
text_set_colors:
    0xFF05 STORE8
    0xFF04 STORE8
    RET
text_home:
    0 0xFF02 STORE8
    0 0xFF03 STORE8
    RET
text_cr:
    0 0xFF02 STORE8
    RET
text_putc:
    0xFF06 STORE8
    RET
text_clear:
    0 0x00F2 STORE8
text_clear_y:
    0x00F2 LOAD8 0xFF03 STORE8
    0 0x00F3 STORE8
text_clear_x:
    0x00F3 LOAD8 0xFF02 STORE8
    32 0xFF06 STORE8
    0x00F3 LOAD8 1 ADD 0x00F3 STORE8
    0x00F3 LOAD8 40 EQ JZ text_clear_x
    0x00F2 LOAD8 1 ADD 0x00F2 STORE8
    0x00F2 LOAD8 25 EQ JZ text_clear_y
    JMP text_home
