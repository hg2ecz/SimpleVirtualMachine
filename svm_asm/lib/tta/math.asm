; TTA arithmetic demo. R0=a,R1=b -> R0=result.
.proc u16_add
 MOV R0,ALU.X
 MOV R1,ALU.ADD
 MOV ALU.OUT,R0
 RET
.endproc
.proc u16_sub
 MOV R0,ALU.X
 MOV R1,ALU.SUB
 MOV ALU.OUT,R0
 RET
.endproc
.proc u16_mul
 MOV R0,ALU.X
 MOV R1,ALU.MUL
 MOV ALU.OUT,R0
 RET
.endproc
.proc u16_div
 MOV R0,ALU.X
 MOV R1,ALU.DIV
 MOV ALU.OUT,R0
 RET
.endproc
.proc u16_mod
 MOV R0,ALU.X
 MOV R1,ALU.MOD
 MOV ALU.OUT,R0
 RET
.endproc
.proc u8_add
 CALL u16_add
 MOV R0,ALU.X
 MOV 0xff,ALU.AND
 MOV ALU.OUT,R0
 RET
.endproc
.proc u8_sub
 CALL u16_sub
 MOV R0,ALU.X
 MOV 0xff,ALU.AND
 MOV ALU.OUT,R0
 RET
.endproc
.proc u8_mul
 CALL u16_mul
 MOV R0,ALU.X
 MOV 0xff,ALU.AND
 MOV ALU.OUT,R0
 RET
.endproc
.proc f16_neg
 MOV R0,ALU.X
 MOV 0x8000,ALU.XOR
 MOV ALU.OUT,R0
 RET
.endproc
.proc f16_abs
 MOV R0,ALU.X
 MOV 0x7fff,ALU.AND
 MOV ALU.OUT,R0
 RET
.endproc
