; TTA16 console helpers. Console byte MMIO = 0xFF20.
; ABI: putc R0=char; puts R0=pointer to NUL-terminated RAM string.
; Clobbers R1 and ALU/MEM ports; puts advances R0.
putc:
    MOV 0xFF20, MEM.ADDR
    MOV R0, MEM.W8
    RET
newline:
    MOV 0xFF20, MEM.ADDR
    MOV 13, MEM.W8
    MOV 10, MEM.W8
    RET
puts:
puts_loop:
    MOV R0, MEM.ADDR
    MOV MEM.R8, R1
    MOV R1, ALU.X
    MOV 0, ALU.CMP
    JZ puts_done
    MOV 0xFF20, MEM.ADDR
    MOV R1, MEM.W8
    MOV R0, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R0
    JMP puts_loop
puts_done:
    RET
