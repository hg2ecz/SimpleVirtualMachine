\ C/ASM bridge uses memory slots even though ordinary Stack C calls use the data stack.
.proc __asm_asm_inc
    __cabi_asm_inc_x @
    1 +
    __cabi_asm_inc_return !
    RET
.endproc
