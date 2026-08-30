; Memory-to-Memory ISA console helpers. Console byte MMIO = 0xFF20.
; ABI: putc A0=address of byte to print; puts A0=pointer to NUL-terminated string.
; Scratch byte: 0x00F0. A0 advances in puts.
putc:
    MOV8 [0xFF20], [A0]
    RET
newline:
    MOV8 [0xFF20], 13
    MOV8 [0xFF20], 10
    RET
puts:
puts_loop:
    MOV8 [0x00F0], [A0+]
    CMP8 [0x00F0], 0
    JZ puts_done
    MOV8 [0xFF20], [0x00F0]
    JMP puts_loop
puts_done:
    RET
