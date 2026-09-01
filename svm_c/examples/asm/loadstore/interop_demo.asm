.proc __asm_asm_inc
    LDI R1, __cabi_asm_inc_x
    LOAD16 R0, [R1]
    ADDI R0, 1
    LDI R1, __cabi_asm_inc_return
    STORE16 [R1], R0
    RET
.endproc
