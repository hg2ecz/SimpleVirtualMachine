\ Stack arithmetic demo. Binary operations: ( a b -- result ).
.proc u8_add
 ADD 0xff AND RET
.endproc
.proc u8_sub
 SUB 0xff AND RET
.endproc
.proc u8_mul
 MUL 0xff AND RET
.endproc
.proc u16_add
 ADD RET
.endproc
.proc u16_sub
 SUB RET
.endproc
.proc u16_mul
 MUL RET
.endproc
.proc u16_div
 DIV RET
.endproc
.proc u16_mod
 MOD RET
.endproc
.proc i16_add
 ADD RET
.endproc
.proc i16_sub
 SUB RET
.endproc
.proc i16_mul
 MUL RET
.endproc
.proc i16_div
 DIV RET
.endproc
.proc f16_neg
 0x8000 XOR RET
.endproc
.proc f16_abs
 0x7fff AND RET
.endproc
