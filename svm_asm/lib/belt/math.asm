; Belt ISA arithmetic demo. b0=a,b1=b -> b0=result.
.proc u8_add
 ADD b0,b1
 LDI 0xff
 AND b1,b0
 RET
.endproc
.proc u8_sub
 SUB b0,b1
 LDI 0xff
 AND b1,b0
 RET
.endproc
.proc u8_mul
 MUL b0,b1
 LDI 0xff
 AND b1,b0
 RET
.endproc
.proc u16_add
 ADD b0,b1
 RET
.endproc
.proc u16_sub
 SUB b0,b1
 RET
.endproc
.proc u16_mul
 MUL b0,b1
 RET
.endproc
.proc u16_div
 DIV b0,b1
 RET
.endproc
.proc u16_mod
 MOD b0,b1
 RET
.endproc
.proc u32_add
 ADD b0,b2
 ADC b1,b3
 RET
.endproc
.proc u32_sub
 SUB b0,b2
 SBC b1,b3
 RET
.endproc
.proc i32_add
 JMP u32_add
.endproc
.proc i32_sub
 JMP u32_sub
.endproc
.proc f16_neg
 LDI 0x8000
 XOR b1,b0
 RET
.endproc
.proc f16_abs
 LDI 0x7fff
 AND b1,b0
 RET
.endproc
