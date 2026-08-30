; Load/Store ISA console helpers. Console byte MMIO = 0xFF20.
; ABI: putc R0=char; puts R0=pointer to NUL-terminated RAM string.
; Clobbers R1,R2.
putc:
    LDI R1, 0xFF20
    STORE8 [R1], R0
    RET
newline:
    LDI R1, 0xFF20
    LDI R0, 13
    STORE8 [R1], R0
    LDI R0, 10
    STORE8 [R1], R0
    RET
puts:
    LDI R2, 0xFF20
puts_loop:
    LOAD8 R1, [R0]
    INC R0
    CMPI R1, 0
    JZ puts_done
    STORE8 [R2], R1
    JMP puts_loop
puts_done:
    RET
