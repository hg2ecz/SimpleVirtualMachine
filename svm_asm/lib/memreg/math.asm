; MemReg arithmetic demo. W=a, file 0xA0=b -> W=result.
.equ MATH_B,0xA0
.proc u8_add
 ADD MATH_B,W
 ANDI 0xff
 RET
.endproc
.proc u8_sub
 SUB MATH_B,W
 ANDI 0xff
 RET
.endproc
.proc u8_mul
 MUL MATH_B,W
 ANDI 0xff
 RET
.endproc
.proc u16_add
 ADD MATH_B,W
 RET
.endproc
.proc u16_sub
 SUB MATH_B,W
 RET
.endproc
.proc u16_mul
 MUL MATH_B,W
 RET
.endproc
.proc u16_div
 DIV MATH_B,W
 RET
.endproc
.proc u16_mod
 MOD MATH_B,W
 RET
.endproc
.proc i16_add
 JMP u16_add
.endproc
.proc i16_sub
 JMP u16_sub
.endproc
.proc i16_mul
 JMP u16_mul
.endproc
.proc i16_div
 JMP u16_div
.endproc
.proc f16_neg
 XORI 0x8000
 RET
.endproc
.proc f16_abs
 ANDI 0x7fff
 RET
.endproc
