; MemReg ISA console helpers. Console byte MMIO = 0xFF20.
; ABI: putc W=char; puts FSR0=pointer to NUL-terminated RAM string.
; FSR1 is used as the console pointer.
putc:
    FSR1I 0xFF20
    STB1
    RET
newline:
    FSR1I 0xFF20
    LDI 13
    STB1
    LDI 10
    STB1
    RET
puts:
    FSR1I 0xFF20
puts_loop:
    LDB0+
    CMPI 0
    JZ puts_done
    STB1
    JMP puts_loop
puts_done:
    RET
