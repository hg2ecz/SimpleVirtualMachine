; Accumulator ISA console helpers. Console byte MMIO = 0xFF20.
; ABI: putc A=char; puts X=pointer to NUL-terminated RAM string.
.proc putc
    STA8 0xFF20
    RET
.endproc

.proc newline
    LDAI 13
    STA8 0xFF20
    LDAI 10
    STA8 0xFF20
    RET
.endproc

.proc puts
puts_loop:
    LDA8 [X+]
    CMPI 0
    JZ puts_done
    STA8 0xFF20
    JMP puts_loop
puts_done:
    RET
.endproc
