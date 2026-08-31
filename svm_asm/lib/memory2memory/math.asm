; Memory-to-Memory arithmetic demo. Inputs/results use fixed scratch words.
.equ MATH_A,0x0100
.equ MATH_B,0x0104
.equ MATH_R,0x0108
.proc u8_add
 MOV8 [MATH_R],[MATH_A]
 ADD8 [MATH_R],[MATH_B]
 RET
.endproc
.proc u8_sub
 MOV8 [MATH_R],[MATH_A]
 SUB8 [MATH_R],[MATH_B]
 RET
.endproc
.proc u16_add
 MOV16 [MATH_R],[MATH_A]
 ADD16 [MATH_R],[MATH_B]
 RET
.endproc
.proc u16_sub
 MOV16 [MATH_R],[MATH_A]
 SUB16 [MATH_R],[MATH_B]
 RET
.endproc
.proc u16_mul
 MOV16 [MATH_R],[MATH_A]
 MUL16 [MATH_R],[MATH_B]
 RET
.endproc
.proc u16_div
 MOV16 [MATH_R],[MATH_A]
 DIV16 [MATH_R],[MATH_B]
 RET
.endproc
.proc u16_mod
 MOV16 [MATH_R],[MATH_A]
 MOD16 [MATH_R],[MATH_B]
 RET
.endproc
.proc u32_add
 MOV16 [MATH_R],[MATH_A]
 MOV16 [MATH_R+2],[MATH_A+2]
 ADD16 [MATH_R],[MATH_B]
 ADC16 [MATH_R+2],[MATH_B+2]
 RET
.endproc
.proc u32_sub
 MOV16 [MATH_R],[MATH_A]
 MOV16 [MATH_R+2],[MATH_A+2]
 SUB16 [MATH_R],[MATH_B]
 SBC16 [MATH_R+2],[MATH_B+2]
 RET
.endproc
.proc f16_neg
 MOV16 [MATH_R],[MATH_A]
 XOR16 [MATH_R],0x8000
 RET
.endproc
.proc f16_abs
 MOV16 [MATH_R],[MATH_A]
 AND16 [MATH_R],0x7fff
 RET
.endproc
