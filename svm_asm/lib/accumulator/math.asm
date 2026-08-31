; Accumulator ISA arithmetic demo. A=a, X=b -> A=result.
.proc u8_add
 ANDI 0xff
 ADDX
 ANDI 0xff
 RET
.endproc
.proc u8_sub
 ANDI 0xff
 SUBX
 ANDI 0xff
 RET
.endproc
.proc u8_mul
 ANDI 0xff
 MULX
 ANDI 0xff
 RET
.endproc
.proc u16_add
 ADDX
 RET
.endproc
.proc u16_sub
 SUBX
 RET
.endproc
.proc u16_mul
 MULX
 RET
.endproc
.proc u16_div
 DIVX
 RET
.endproc
.proc u16_mod
 MODX
 RET
.endproc
.proc i16_add
 ADDX
 RET
.endproc
.proc i16_sub
 SUBX
 RET
.endproc
.proc i16_mul
 MULX
 RET
.endproc
.proc i16_div
 DIVX
 RET
.endproc
.proc f16_neg
 XORI 0x8000
 RET
.endproc
.proc f16_abs
 ANDI 0x7fff
 RET
.endproc
.proc f32_neg
 TXA
 XORI 0x8000
 TAX
 RET
.endproc
.proc f32_abs
 TXA
 ANDI 0x7fff
 TAX
 RET
.endproc
