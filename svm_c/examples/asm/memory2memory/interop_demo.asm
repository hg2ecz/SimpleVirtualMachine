.proc __asm_asm_inc
    MOV16 [__cabi_asm_inc_return], [__cabi_asm_inc_x]
    ADD16 [__cabi_asm_inc_return], 1
    RET
.endproc
