.proc __asm_asm_inc
    LD16A __cabi_asm_inc_x
    LDI 1
    ADD b1,b0
    ST16A __cabi_asm_inc_return,b0
    RET
.endproc
