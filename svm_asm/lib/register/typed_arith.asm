; Typed arithmetic reference library for the Register ISA.
; Educational/demo counterpart of svm_c/lib/{arithmetic_int,wide_int,f16,f32}.sc.
; Integer scalar ABI: R0=a, R1=b -> R0=result unless noted.
; 32-bit ABI: a=R1:R0, b=R3:R2 -> result=R1:R0 (hi:lo).
; f16 uses IEEE-754 binary16 bit patterns in R0/R1. Full portable soft-float
; remains canonical in SVM-C; this file demonstrates scalar helpers and the
; multiword primitives needed by generated/hand-written soft-float code.

.proc u8_add
    ADD R0,R1
    MOVI R7,0x00ff
    AND R0,R7
    RET
.endproc
.proc u8_sub
    SUB R0,R1
    MOVI R7,0x00ff
    AND R0,R7
    RET
.endproc
.proc u8_mul
    MUL R0,R1
    MOVI R7,0x00ff
    AND R0,R7
    RET
.endproc
.proc u8_div
    MOVI R7,0x00ff
    AND R0,R7
    AND R1,R7
    DIV R0,R1
    RET
.endproc
.proc u8_mod
    MOVI R7,0x00ff
    AND R0,R7
    AND R1,R7
    MOD R0,R1
    RET
.endproc

; Sign-extend low byte in R0.
.proc i8_sext
    MOVI R7,0x00ff
    AND R0,R7
    MOVI R7,0x0080
    AND R7,R0
    CMPI R7,0
    JZ i8_sext_done
    MOVI R7,0xff00
    OR R0,R7
i8_sext_done:
    RET
.endproc
.proc i8_add
    CALL i8_sext
    PUSH R0
    MOV R0,R1
    CALL i8_sext
    MOV R1,R0
    POP R0
    ADD R0,R1
    CALL i8_sext
    RET
.endproc
.proc i8_sub
    CALL i8_sext
    PUSH R0
    MOV R0,R1
    CALL i8_sext
    MOV R1,R0
    POP R0
    SUB R0,R1
    CALL i8_sext
    RET
.endproc
.proc i8_mul
    CALL i8_sext
    PUSH R0
    MOV R0,R1
    CALL i8_sext
    MOV R1,R0
    POP R0
    MUL R0,R1
    CALL i8_sext
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
.proc i16_add
    ADD R0,R1
    RET
.endproc
.proc i16_sub
    SUB R0,R1
    RET
.endproc
.proc i16_mul
    MUL R0,R1
    RET
.endproc
; Signed division: truncation toward zero. R0/R1 -> R0.
.proc i16_div
    MOVI R6,0
    CMPI R0,0
    JNN i16_div_a_pos
    NEG R0
    MOVI R6,1
i16_div_a_pos:
    CMPI R1,0
    JNN i16_div_b_pos
    NEG R1
    MOVI R7,1
    XOR R6,R7
i16_div_b_pos:
    DIV R0,R1
    CMPI R6,0
    JZ i16_div_done
    NEG R0
i16_div_done:
    RET
.endproc

; 32-bit wraparound add/sub.
.proc u32_add
    ADD R0,R2
    ADC R1,R3
    RET
.endproc
.proc i32_add
    ADD R0,R2
    ADC R1,R3
    RET
.endproc
.proc u32_sub
    SUB R0,R2
    SBC R1,R3
    RET
.endproc
.proc i32_sub
    SUB R0,R2
    SBC R1,R3
    RET
.endproc

; 32x32 -> low 32 bits. Uses R4..R7.
.proc u32_mul
    MOV R4,R0
    MOV R5,R2
    MULHU R4,R5          ; high(a_lo*b_lo)
    MOV R6,R1
    MUL R6,R2            ; a_hi*b_lo low16
    ADD R4,R6
    MOV R6,R0
    MUL R6,R3            ; a_lo*b_hi low16
    ADD R4,R6
    MUL R0,R2            ; low product
    MOV R1,R4
    RET
.endproc
.proc i32_mul
    JMP u32_mul           ; modulo-2^32 product is representation-independent
.endproc

; Exact 16x16 -> 32 product: R0=a,R1=b -> R1:R0.
.proc u16_mul_u32
    MOV R2,R0
    MULHU R2,R1
    MUL R0,R1
    MOV R1,R2
    RET
.endproc

; f16 representation helpers (IEEE binary16 bit patterns).
.proc f16_neg
    MOVI R1,0x8000
    XOR R0,R1
    RET
.endproc
.proc f16_abs
    MOVI R1,0x7fff
    AND R0,R1
    RET
.endproc
.proc f16_is_zero
    MOVI R1,0x7fff
    AND R0,R1
    CMPI R0,0
    JZ f16_is_zero_yes
    MOVI R0,0
    RET
f16_is_zero_yes:
    MOVI R0,1
    RET
.endproc
.proc f16_is_inf
    MOVI R1,0x7fff
    AND R0,R1
    CMPI R0,0x7c00
    JZ f16_is_inf_yes
    MOVI R0,0
    RET
f16_is_inf_yes:
    MOVI R0,1
    RET
.endproc
.proc f16_is_nan
    MOV R2,R0
    MOVI R1,0x7c00
    AND R0,R1
    CMPI R0,0x7c00
    JNZ f16_is_nan_no
    MOV R0,R2
    MOVI R1,0x03ff
    AND R0,R1
    CMPI R0,0
    JZ f16_is_nan_no
    MOVI R0,1
    RET
f16_is_nan_no:
    MOVI R0,0
    RET
.endproc

; Hardware has no FPU. f16_add/sub/mul/div and f32_* are intentionally kept
; canonical in svm_c/lib/f16.sc and f32.sc. The integer helpers above are the
; assembly primitives a generated hand-optimized soft-float implementation uses.
