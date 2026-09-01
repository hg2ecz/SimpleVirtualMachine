.proc __asm_asm_inc
    MOV __cabi_asm_inc_x,MEM.ADDR
    MOV MEM.R16,R0
    MOV R0,ALU.X
    MOV 1,ALU.ADD
    MOV ALU.OUT,R0
    MOV __cabi_asm_inc_return,MEM.ADDR
    MOV R0,MEM.W16
    RET
.endproc
