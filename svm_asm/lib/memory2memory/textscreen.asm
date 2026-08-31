; 40x25 framebuffer text-screen helpers.
; text_goto: x byte at [A0], y byte at [A1]. text_set_colors: fg [A0], bg [A1].
; text_putc: character byte at [A0]. Scratch: 0x00F2..0x00F3.
.proc text_goto
    MOV8 [0xFF02], [A0]
    MOV8 [0xFF03], [A1]
    RET
.endproc

.proc text_set_colors
    MOV8 [0xFF04], [A0]
    MOV8 [0xFF05], [A1]
    RET
.endproc

.proc text_home
    MOV8 [0xFF02], 0
    MOV8 [0xFF03], 0
    RET
.endproc

.proc text_cr
    MOV8 [0xFF02], 0
    RET
.endproc

.proc text_putc
    MOV8 [0xFF06], [A0]
    RET
.endproc

.proc text_clear
    MOV8 [0x00F2], 0
text_clear_y:
    MOV8 [0xFF03], [0x00F2]
    MOV8 [0x00F3], 0
text_clear_x:
    MOV8 [0xFF02], [0x00F3]
    MOV8 [0xFF06], 32
    INC8 [0x00F3]
    CMP8 [0x00F3], 40
    JNZ text_clear_x
    INC8 [0x00F2]
    CMP8 [0x00F2], 25
    JNZ text_clear_y
    JMP text_home
.endproc
