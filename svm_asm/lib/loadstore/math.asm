; Load/Store arithmetic demo. R0=a,R1=b -> R0=result.
.proc u8_add
 ADD R0,R1
 ANDI R0,0xff
 RET
.endproc
.proc u8_sub
 SUB R0,R1
 ANDI R0,0xff
 RET
.endproc
.proc u8_mul
 MUL R0,R1
 ANDI R0,0xff
 RET
.endproc
.proc u16_add
 ADD R0,R1
 RET
.endproc
.proc u16_sub
 SUB R0,R1
 RET
.endproc
.proc u16_mul
 MUL R0,R1
 RET
.endproc
.proc u16_div
 DIV R0,R1
 RET
.endproc
.proc u16_mod
 MOD R0,R1
 RET
.endproc
.proc u32_add
 ADD R0,R2
 ADC R1,R3
 RET
.endproc
.proc u32_sub
 SUB R0,R2
 SBC R1,R3
 RET
.endproc
.proc i32_add
 JMP u32_add
.endproc
.proc i32_sub
 JMP u32_sub
.endproc
.proc f16_neg
 XORI R0,0x8000
 RET
.endproc
.proc f16_abs
 ANDI R0,0x7fff
 RET
.endproc
.proc f32_neg
 XORI R1,0x8000
 RET
.endproc
.proc f32_abs
 ANDI R1,0x7fff
 RET
.endproc
