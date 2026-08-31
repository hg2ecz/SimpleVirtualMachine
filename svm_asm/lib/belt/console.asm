; Belt16 console helpers. Console byte MMIO = 0xFF20.
; ABI: putc b0=char; puts b0=pointer to NUL-terminated RAM string.
; puts uses zero-page 0x00F0 as its pointer scratch.
.proc putc
    ST8A 0xFF20,b0
    RET
.endproc

.proc newline
    LDI 13
    ST8A 0xFF20,b0
    LDI 10
    ST8A 0xFF20,b0
    RET
.endproc

.proc puts
    ZST16 0xF0,b0
puts_loop:
    ZLD16 0xF0
    LD8 [b0]
    LDI 0
    CMP b1,b0
    JZ puts_done
    ST8A 0xFF20,b2
    LDI 1
    ADD b4,b0
    ZST16 0xF0,b0
    JMP puts_loop
puts_done:
    RET
.endproc
