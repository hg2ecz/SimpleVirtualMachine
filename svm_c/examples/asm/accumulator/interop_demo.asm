.proc __asm_asm_inc
    LDA16 __cabi_asm_inc_x
    INC
    STA16 __cabi_asm_inc_return
    RET
.endproc
