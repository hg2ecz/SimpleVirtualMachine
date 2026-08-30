; 40x25 framebuffer text-screen helpers. ABI: R0=x/fg/char, R1=y/bg.
text_goto:
    MOVI R2, 0xFF02
    STORE8 [R2], R0
    MOVI R2, 0xFF03
    STORE8 [R2], R1
    RET
text_set_colors:
    MOVI R2, 0xFF04
    STORE8 [R2], R0
    MOVI R2, 0xFF05
    STORE8 [R2], R1
    RET
text_home:
    MOVI R0, 0
    MOVI R1, 0
    JMP text_goto
text_cr:
    MOVI R2, 0xFF02
    MOVI R0, 0
    STORE8 [R2], R0
    RET
text_putc:
    MOVI R2, 0xFF06
    STORE8 [R2], R0
    RET
text_clear:
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
