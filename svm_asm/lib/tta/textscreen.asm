; 40x25 framebuffer text-screen helpers. ABI: R0=x/fg/char, R1=y/bg.
.proc text_goto
    MOV 0xFF02, MEM.ADDR
    MOV R0, MEM.W8
    MOV 0xFF03, MEM.ADDR
    MOV R1, MEM.W8
    RET
.endproc

.proc text_set_colors
    MOV 0xFF04, MEM.ADDR
    MOV R0, MEM.W8
    MOV 0xFF05, MEM.ADDR
    MOV R1, MEM.W8
    RET
.endproc

.proc text_home
    MOV 0, R0
    MOV 0, R1
    JMP text_goto
.endproc

.proc text_cr
    MOV 0xFF02, MEM.ADDR
    MOV 0, MEM.W8
    RET
.endproc

.proc text_putc
    MOV 0xFF06, MEM.ADDR
    MOV R0, MEM.W8
    RET
.endproc

.proc text_clear
    MOV 0, R0
text_clear_y:
    MOV 0xFF03, MEM.ADDR
    MOV R0, MEM.W8
    MOV 0, R1
text_clear_x:
    MOV 0xFF02, MEM.ADDR
    MOV R1, MEM.W8
    MOV 0xFF06, MEM.ADDR
    MOV 32, MEM.W8
    MOV R1, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R1
    MOV R1, ALU.X
    MOV 40, ALU.CMP
    JNZ text_clear_x
    MOV R0, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R0
    MOV R0, ALU.X
    MOV 25, ALU.CMP
    JNZ text_clear_y
    JMP text_home
.endproc
