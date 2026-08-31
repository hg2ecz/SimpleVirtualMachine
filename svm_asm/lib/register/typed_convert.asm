.include "typed_arith.asm"
; Typed string conversion reference library for Register ISA.
; NUL-terminated ASCII. For compactness the assembly demo provides complete
; 8/16-bit integer dec/hex/bin conversion and parse helpers plus raw f16/f32
; hexadecimal representation helpers. Full numeric f16/f32 decimal conversion
; stays in the portable C library.

.proc asm_digit_value
    ; R0=ASCII -> R0=0..15 or ffff
    CMPI R0,48
    JC asm_digit_bad
    CMPI R0,58
    JC asm_digit_dec
    CMPI R0,65
    JC asm_digit_lower_check
    CMPI R0,71
    JC asm_digit_upper
asm_digit_lower_check:
    CMPI R0,97
    JC asm_digit_bad
    CMPI R0,103
    JC asm_digit_lower
asm_digit_bad:
    MOVI R0,0xffff
    RET
asm_digit_dec:
    SUBI R0,48
    RET
asm_digit_upper:
    SUBI R0,55
    RET
asm_digit_lower:
    SUBI R0,87
    RET
.endproc

.proc asm_hexchar
    ; R0 low nibble -> ASCII
    MOVI R1,15
    AND R0,R1
    CMPI R0,10
    JC asm_hexchar_dec
    ADDI R0,55
    RET
asm_hexchar_dec:
    ADDI R0,48
    RET
.endproc

; R0=dst, R1=value -> R0=original dst. 4 digits + NUL.
.proc u16_to_hexstr
    MOV R6,R0
    MOV R5,R0
    MOV R4,R1
    MOV R0,R4
    MOVI R2,12
    SHR R0,R2
    CALL asm_hexchar
    STORE8 [R5+],R0
    MOV R0,R4
    MOVI R2,8
    SHR R0,R2
    CALL asm_hexchar
    STORE8 [R5+],R0
    MOV R0,R4
    MOVI R2,4
    SHR R0,R2
    CALL asm_hexchar
    STORE8 [R5+],R0
    MOV R0,R4
    CALL asm_hexchar
    STORE8 [R5+],R0
    MOVI R0,0
    STORE8 [R5],R0
    MOV R0,R6
    RET
.endproc

; R0=dst,R1=value -> exactly 16 binary digits + NUL.
.proc u16_to_binstr
    MOV R6,R0
    MOV R5,R0
    MOV R4,R1
    MOVI R3,16
    MOVI R2,0x8000
u16_to_binstr_loop:
    MOV R0,R4
    AND R0,R2
    CMPI R0,0
    JZ u16_to_binstr_zero
    MOVI R0,49
    JMP u16_to_binstr_store
u16_to_binstr_zero:
    MOVI R0,48
u16_to_binstr_store:
    STORE8 [R5+],R0
    SHR1 R2
    DEC R3
    JNZ u16_to_binstr_loop
    MOVI R0,0
    STORE8 [R5],R0
    MOV R0,R6
    RET
.endproc

; R0=dst,R1=value -> unsigned decimal + NUL. Buffer >=6 bytes.
.proc u16_to_decstr
    MOV R6,R0
    MOV R5,R0
    MOV R4,R1
    CMPI R4,0
    JNZ u16_to_decstr_nonzero
    MOVI R0,48
    STORE8 [R5+],R0
    MOVI R0,0
    STORE8 [R5],R0
    MOV R0,R6
    RET
u16_to_decstr_nonzero:
    MOVI R3,0
u16_to_decstr_div:
    MOV R0,R4
    MOVI R1,10
    MOD R0,R1
    ADDI R0,48
    PUSH R0
    MOV R0,R4
    DIV R0,R1
    MOV R4,R0
    INC R3
    CMPI R4,0
    JNZ u16_to_decstr_div
u16_to_decstr_emit:
    POP R0
    STORE8 [R5+],R0
    DEC R3
    JNZ u16_to_decstr_emit
    MOVI R0,0
    STORE8 [R5],R0
    MOV R0,R6
    RET
.endproc

; signed decimal, R1 contains two's-complement i16.
.proc i16_to_decstr
    CMPI R1,0
    JNN i16_to_decstr_pos
    MOV R5,R0
    MOVI R2,45
    STORE8 [R5+],R2
    NEG R1
    MOV R0,R5
    CALL u16_to_decstr
    RET
i16_to_decstr_pos:
    JMP u16_to_decstr
.endproc

; u8/i8 wrappers normalize then use 16-bit routines.
.proc u8_to_decstr
    MOVI R2,0x00ff
    AND R1,R2
    JMP u16_to_decstr
.endproc
.proc i8_to_decstr
    MOV R2,R0
    MOV R0,R1
    CALL i8_sext
    MOV R1,R0
    MOV R0,R2
    JMP i16_to_decstr
.endproc
.proc u8_to_hexstr
    ; still writes four digits by design; callers wanting 2 can skip leading bytes.
    MOVI R2,0x00ff
    AND R1,R2
    JMP u16_to_hexstr
.endproc
.proc u8_to_binstr
    MOVI R2,0x00ff
    AND R1,R2
    JMP u16_to_binstr
.endproc

; R0=src -> R0=u16, stops at first non-decimal digit.
.proc parse_u16_decstr
    MOV R5,R0
    MOVI R4,0
parse_u16_decstr_loop:
    LOAD8 R0,[R5+]
    CALL asm_digit_value
    CMPI R0,10
    JNC parse_u16_decstr_done
    MOV R3,R0
    MOV R0,R4
    MOVI R1,10
    MUL R0,R1
    ADD R0,R3
    MOV R4,R0
    JMP parse_u16_decstr_loop
parse_u16_decstr_done:
    MOV R0,R4
    RET
.endproc

; R0=src -> R0=u16, accepts optional 0x/0X.
.proc parse_u16_hexstr
    MOV R5,R0
    MOVI R4,0
parse_u16_hexstr_loop:
    LOAD8 R0,[R5+]
    CALL asm_digit_value
    CMPI R0,16
    JNC parse_u16_hexstr_done
    MOV R3,R0
    MOV R0,R4
    MOVI R1,4
    SHL R0,R1
    OR R0,R3
    MOV R4,R0
    JMP parse_u16_hexstr_loop
parse_u16_hexstr_done:
    MOV R0,R4
    RET
.endproc

; R0=src -> R0=u16, accepts sequence of 0/1.
.proc parse_u16_binstr
    MOV R5,R0
    MOVI R4,0
parse_u16_binstr_loop:
    LOAD8 R0,[R5+]
    CMPI R0,48
    JZ parse_u16_binstr_bit0
    CMPI R0,49
    JNZ parse_u16_binstr_done
    MOVI R3,1
    JMP parse_u16_binstr_shift
parse_u16_binstr_bit0:
    MOVI R3,0
parse_u16_binstr_shift:
    MOV R0,R4
    SHL1 R0
    OR R0,R3
    MOV R4,R0
    JMP parse_u16_binstr_loop
parse_u16_binstr_done:
    MOV R0,R4
    RET
.endproc

; f16 is one 16-bit IEEE bit pattern: hex conversion is directly reusable.
.proc f16_to_hexstr
    JMP u16_to_hexstr
.endproc
.proc parse_f16_hexstr
    JMP parse_u16_hexstr
.endproc

; f32 raw representation helper. ABI R0=dst, R1=lo16, R2=hi16.
; Writes 8 hex digits (high word first) + NUL.
.proc f32_to_hexstr
    MOV R6,R0
    PUSH R1
    MOV R1,R2
    CALL u16_to_hexstr
    ; overwrite NUL at +4 with low word
    MOV R0,R6
    ADDI R0,4
    POP R1
    CALL u16_to_hexstr
    MOV R0,R6
    RET
.endproc
