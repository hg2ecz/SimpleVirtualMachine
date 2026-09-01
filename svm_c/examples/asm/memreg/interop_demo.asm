.proc __asm_asm_inc
    FSR0I __cabi_asm_inc_x
    LDW0
    ADDI 1
    FSR0I __cabi_asm_inc_return
    STW0
    RET
.endproc
