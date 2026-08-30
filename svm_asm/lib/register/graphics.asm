; Register ISA 320x200x2-bpp graphics helpers.
; Pixel values are palette slots 0..3; slots map through MMIO 0xFF0C..0xFF0F.
; Scratch RAM: 0x00E8 holds current color.
; ABI:
;   gfx_set_color: R0=color 0..3
;   gfx_set_palette: R0..R3=master palette indexes 0..15
;   putpixel: R0=x (0..319), R1=y (0..199), uses current color
;   clear: R0=color 0..3
;   hline: R0=x0, R1=x1, R2=y; current color
;   vline: R0=x, R1=y0, R2=y1; current color
; putpixel assumes in-range coordinates. Clobbers R0..R7.

gfx_set_color:
    MOVI R1, 3
    AND R0, R1
    MOVI R1, 0x00E8
    STORE8 [R1], R0
    RET

gfx_set_palette:
    MOVI R4, 0xFF0C
    STORE8 [R4+], R0
    STORE8 [R4+], R1
    STORE8 [R4+], R2
    STORE8 [R4], R3
    RET

putpixel:
    ; byte address = y*80 + x/4
    MOV R2, R1
    MOVI R3, 80
    MUL R2, R3
    MOV R4, R0
    MOVI R5, 2
    SHR R4, R5
    ADD R2, R4

    ; shift = 6 - 2*(x&3)
    MOV R4, R0
    MOVI R5, 3
    AND R4, R5
    MOVI R5, 1
    SHL R4, R5
    MOVI R5, 6
    SUB R5, R4

    ; mask = 3 << shift
    MOVI R3, 3
    SHL R3, R5
    VLOAD8 R4, [R2]
    MOV R7, R3
    NOT R7
    AND R4, R7

    ; insert current slot
    MOVI R6, 0x00E8
    LOAD8 R6, [R6]
    SHL R6, R5
    OR R4, R6
    VSTORE8 [R2], R4
    RET

clear:
    MOVI R1, 3
    AND R0, R1
    MOVI R1, 0x55
    MUL R0, R1
    MOVI R1, 0
    MOVI R2, 16000
clear_loop:
    VSTORE8P [R1+], R0
    DEC R2
    JNZ clear_loop
    RET

hline:
    ; preserve loop state around putpixel, which clobbers all work registers
    MOV R4, R0
    MOV R5, R1
    MOV R6, R2
hline_loop:
    PUSH R4
    PUSH R5
    PUSH R6
    MOV R0, R4
    MOV R1, R6
    CALL putpixel
    POP R6
    POP R5
    POP R4
    CMP R4, R5
    JZ hline_done
    INC R4
    JMP hline_loop
hline_done:
    RET

vline:
    MOV R4, R0
    MOV R5, R1
    MOV R6, R2
vline_loop:
    PUSH R4
    PUSH R5
    PUSH R6
    MOV R0, R4
    MOV R1, R5
    CALL putpixel
    POP R6
    POP R5
    POP R4
    CMP R5, R6
    JZ vline_done
    INC R5
    JMP vline_loop
vline_done:
    RET
