; MemReg ISA 320x200x2-bpp graphics helpers.
; File scratch 0xE8..0xF8. ABI: gfx_set_color W=color;
; gfx_set_palette FSR0=pointer to four palette bytes; putpixel FSR0=x,FSR1=y;
; clear W=color; hline FSR0=x0,FSR1=x1,W=y; vline FSR0=x,FSR1=y0,W=y1.
.proc gfx_set_color
    ANDI 3
    MOV8 W,0xE8
    RET
.endproc

.proc gfx_set_palette
    FSR1I 0xFF0C
    LDB0+
    STB1+
    LDB0+
    STB1+
    LDB0+
    STB1+
    LDB0+
    STB1+
    RET
.endproc

.proc putpixel
    F02W
    MOV16 W,0xEA
    F12W
    MOV16 W,0xEC
    LDI 80
    MOV16 W,0xF0
    MOV16 0xEC,W
    MUL 0xF0,W
    MOV16 W,0xEE
    MOV16 0xEA,W
    LDI 2
    MOV16 W,0xF0
    MOV16 0xEA,W
    SHR 0xF0,W
    ADD 0xEE,W
    MOV16 W,0xEE
    W2F0
    VLDB0
    MOV16 W,0xF2
    MOV16 0xEA,W
    ANDI 3
    SHL1W
    MOV16 W,0xF4
    LDI 6
    SUB 0xF4,W
    MOV16 W,0xF4
    LDI 3
    MOV16 W,0xF6
    MOV16 0xF4,W
    MOV16 W,0xF0
    MOV16 0xF6,W
    SHL 0xF0,W
    NOTW
    AND 0xF2,W
    MOV16 W,0xF2
    MOV8 0xE8,W
    MOV16 0xF4,W
    MOV16 W,0xF0
    MOV8 0xE8,W
    SHL 0xF0,W
    OR 0xF2,W
    MOV16 W,0xF2
    MOV16 0xEE,W
    W2F0
    MOV16 0xF2,W
    VSTB0
    RET
.endproc

.proc clear
    ANDI 3
    MOV16 W,0xEA
    LDI 0x55
    MOV16 W,0xEC
    MOV16 0xEA,W
    MUL 0xEC,W
    MOV16 W,0xEA
    LDI 0
    MOV16 W,0xEC
    LDI 16000
    MOV16 W,0xEE
clear_loop:
    MOV16 0xEC,W
    W2F0
    MOV16 0xEA,W
    VSTB0
    INC 0xEC
    DEC 0xEE
    MOV16 0xEE,W
    CMPI 0
    JNZ clear_loop
    RET
.endproc

.proc hline
    MOV16 W,0xEE
    F02W
    MOV16 W,0xEA
    F12W
    MOV16 W,0xEC
hline_loop:
    MOV16 0xEA,W
    W2F0
    MOV16 0xEE,W
    W2F1
    CALL putpixel
    MOV16 0xEA,W
    CMP 0xEC
    JZ hline_done
    INC 0xEA
    JMP hline_loop
hline_done:
    RET
.endproc

.proc vline
    MOV16 W,0xEE
    F02W
    MOV16 W,0xEA
    F12W
    MOV16 W,0xEC
vline_loop:
    MOV16 0xEA,W
    W2F0
    MOV16 0xEC,W
    W2F1
    CALL putpixel
    MOV16 0xEC,W
    CMP 0xEE
    JZ vline_done
    INC 0xEC
    JMP vline_loop
vline_done:
    RET
.endproc

; ---- High-level geometry -------------------------------------------------
; Shared parameter block for calls that do not fit naturally in registers.
; Including graphics.asm reserves 0x00B0..0x00FA for graphics parameters,
; current colour and internal scratch.
.equ GFX_X0,     0x00C0
.equ GFX_Y0,     0x00C2
.equ GFX_X1,     0x00C4
.equ GFX_Y1,     0x00C6
.equ GFX_W,      0x00C8
.equ GFX_H,      0x00CA
.equ GFX_R,      0x00CC
.equ GFX_COLOR,  0x00CE
.equ GFX_I,      0x00D0
.equ GFX_DX,     0x00D2
.equ GFX_DY,     0x00D4
.equ GFX_SX,     0x00D6
.equ GFX_SY,     0x00D8
.equ GFX_TMP0,   0x00DA
.equ GFX_TMP1,   0x00DC
.equ GFX_TMP2,   0x00DE
.equ GFX_TMP3,   0x00E0
.equ GFX_TMP4,   0x00E2
.equ GFX_TMP5,   0x00E4
.equ GFX_TMP6,   0x00E6

; line: integer DDA, screen-coordinate domain. Parameters:
; GFX_X0=x0, GFX_Y0=y0, GFX_X1=x1, GFX_Y1=y1, GFX_COLOR=color.
; Products are bounded by 319*199 and fit in 16 bits for in-screen points.
.proc line
    MOV16 GFX_COLOR,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    CALL gfx_set_color

    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_X1,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 0xBA,W
    CMP 0xB8
    JNC line_x_negative
    MOV16 0xBA,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    SUB 0xB8,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_DX
    LDI 1
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_SX
    JMP line_x_ready
line_x_negative:
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    SUB 0xBA,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_DX
    LDI 0
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_SX
line_x_ready:
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_Y1,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 0xBA,W
    CMP 0xB8
    JNC line_y_negative
    MOV16 0xBA,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    SUB 0xB8,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_DY
    LDI 1
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_SY
    JMP line_y_ready
line_y_negative:
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    SUB 0xBA,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_DY
    LDI 0
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_SY
line_y_ready:
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP0
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP1
    LDI 0
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_I

    MOV16 GFX_DX,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_DY,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JNC line_vertical_major

line_horizontal_loop:
    ; y = y0 +/- (i*dy/dx), x is maintained in TMP0.
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_DY,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    MOV16 0xBA,W
    MUL 0xBC,W
    MOV16 W,0xBA
    ; dx is non-zero here unless this is a single pixel.
    ; Handle dx=0 explicitly to avoid a DIV-by-zero trap.
    MOV16 GFX_DX,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    LDI 0
    MOV16 W,0xBE
    MOV16 0xBC,W
    CMP 0xBE
    JZ line_single_pixel
    MOV16 0xBA,W
    DIV 0xBC,W
    MOV16 W,0xBA
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB2
    MOV16 GFX_SY,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    LDI 0
    MOV16 W,0xBE
    MOV16 0xBC,W
    CMP 0xBE
    JZ line_h_sub_y
    MOV16 0xB2,W
    ADD 0xBA,W
    MOV16 W,0xB2
    JMP line_h_y_ready
line_h_sub_y:
    MOV16 0xB2,W
    SUB 0xBA,W
    MOV16 W,0xB2
line_h_y_ready:
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    CALL putpixel

    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_DX,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JZ line_done
    MOV16 GFX_SX,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    LDI 0
    MOV16 W,0xBE
    MOV16 0xBC,W
    CMP 0xBE
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    JZ line_h_dec_x
    INC 0xB0
    JMP line_h_store_x
line_h_dec_x:
    DEC 0xB0
line_h_store_x:
    MOV16 0xB0,W
    MOV16 W,GFX_TMP0
    MOV16 GFX_I,W
    MOV16 W,0xB0
    INC 0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_I
    JMP line_horizontal_loop

line_vertical_major:
    MOV16 GFX_DY,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    LDI 0
    MOV16 W,0xBE
    MOV16 0xBC,W
    CMP 0xBE
    JZ line_single_pixel
line_vertical_loop:
    ; x = x0 +/- (i*dx/dy), y is maintained in TMP1.
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_DX,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    MOV16 0xBA,W
    MUL 0xBC,W
    MOV16 W,0xBA
    MOV16 GFX_DY,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    MOV16 0xBA,W
    DIV 0xBC,W
    MOV16 W,0xBA
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_SX,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    LDI 0
    MOV16 W,0xBE
    MOV16 0xBC,W
    CMP 0xBE
    JZ line_v_sub_x
    MOV16 0xB8,W
    ADD 0xBA,W
    MOV16 W,0xB8
    JMP line_v_x_ready
line_v_sub_x:
    MOV16 0xB8,W
    SUB 0xBA,W
    MOV16 W,0xB8
line_v_x_ready:
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 GFX_TMP1,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB2
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    CALL putpixel

    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_DY,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JZ line_done
    MOV16 GFX_SY,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    LDI 0
    MOV16 W,0xBE
    MOV16 0xBC,W
    CMP 0xBE
    MOV16 GFX_TMP1,W
    MOV16 W,0xB0
    JZ line_v_dec_y
    INC 0xB0
    JMP line_v_store_y
line_v_dec_y:
    DEC 0xB0
line_v_store_y:
    MOV16 0xB0,W
    MOV16 W,GFX_TMP1
    MOV16 GFX_I,W
    MOV16 W,0xB0
    INC 0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_I
    JMP line_vertical_loop

line_single_pixel:
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB2
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    CALL putpixel
line_done:
    RET
.endproc

; rect: GFX_X0=x, GFX_Y0=y, GFX_W=w, GFX_H=h, GFX_COLOR=color.
.proc rect
    MOV16 GFX_COLOR,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    CALL gfx_set_color
    MOV16 GFX_W,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    LDI 0
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JZ rect_done
    MOV16 GFX_H,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 0xB8,W
    CMP 0xBA
    JZ rect_done

    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_W,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    DEC 0xBA
    MOV16 0xBA,W
    ADD 0xB8,W
    MOV16 W,0xBA
    MOV16 0xBA,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP0
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_H,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    DEC 0xBA
    MOV16 0xBA,W
    ADD 0xB8,W
    MOV16 W,0xBA
    MOV16 0xBA,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP1

    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB2
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB4
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    MOV16 0xB4,W
    CALL hline

    MOV16 GFX_H,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    LDI 1
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JZ rect_sides
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB2
    MOV16 GFX_TMP1,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB4
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    MOV16 0xB4,W
    CALL hline
rect_sides:
    MOV16 GFX_H,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    LDI 2
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JNC rect_done
    JZ rect_done
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    INC 0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_TMP1,W
    MOV16 W,0xB0
    DEC 0xB0
    MOV16 0xB0,W
    MOV16 W,0xB4
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    MOV16 0xB4,W
    CALL vline
    MOV16 GFX_W,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    LDI 1
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JZ rect_done
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    INC 0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_TMP1,W
    MOV16 W,0xB0
    DEC 0xB0
    MOV16 0xB0,W
    MOV16 W,0xB4
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    MOV16 0xB4,W
    CALL vline
rect_done:
    RET
.endproc

; fillrect: same parameter block as rect.
.proc fillrect
    MOV16 GFX_COLOR,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    CALL gfx_set_color
    MOV16 GFX_W,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    LDI 0
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JZ fillrect_done
    MOV16 GFX_H,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 0xB8,W
    CMP 0xBA
    JZ fillrect_done
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_W,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    DEC 0xBA
    MOV16 0xBA,W
    ADD 0xB8,W
    MOV16 W,0xBA
    MOV16 0xBA,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP0
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP1
    MOV16 GFX_H,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP2
fillrect_loop:
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB2
    MOV16 GFX_TMP1,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB4
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    MOV16 0xB4,W
    CALL hline
    MOV16 GFX_TMP2,W
    MOV16 W,0xB0
    DEC 0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP2
    MOV16 0xB0,W
    MOV16 W,0xB8
    LDI 0
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JZ fillrect_done
    MOV16 GFX_TMP1,W
    MOV16 W,0xB0
    INC 0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP1
    JMP fillrect_loop
fillrect_done:
    RET
.endproc

; Internal helper: plot eight symmetric points. Requires TMP0=x, I=y,
; X0=cx, Y0=cy and current color already selected. Circle must fit on screen.
.proc __gfx_circle_points
    ; (cx+x, cy+y)
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB8,W
    ADD 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    ADD 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    CALL putpixel
    ; (cx+y, cy+x)
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB8,W
    ADD 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    ADD 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    CALL putpixel
    ; (cx-y, cy+x)
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB8,W
    SUB 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    ADD 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    CALL putpixel
    ; (cx-x, cy+y)
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB8,W
    SUB 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    ADD 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    CALL putpixel
    ; (cx-x, cy-y)
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB8,W
    SUB 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    SUB 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    CALL putpixel
    ; (cx-y, cy-x)
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB8,W
    SUB 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    SUB 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    CALL putpixel
    ; (cx+y, cy-x)
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB8,W
    ADD 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    SUB 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    CALL putpixel
    ; (cx+x, cy-y)
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB8,W
    ADD 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    SUB 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    CALL putpixel
    RET
.endproc

; circle: GFX_X0=cx, GFX_Y0=cy, GFX_R=r, GFX_COLOR=color.
; Uses an integer scan of the first octant; circle must fit on screen.
.proc circle
    MOV16 GFX_COLOR,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    CALL gfx_set_color
    MOV16 GFX_R,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 0xB8,W
    MUL 0xB8,W
    MOV16 W,0xB8
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP1
    LDI 0
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_I
circle_y_loop:
circle_find_x:
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 0xB8,W
    MUL 0xB8,W
    MOV16 W,0xB8
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 0xBA,W
    MUL 0xBA,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    ADD 0xBA,W
    MOV16 W,0xB8
    MOV16 GFX_TMP1,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JNC circle_x_ready
    JZ circle_x_ready
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    DEC 0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP0
    JMP circle_find_x
circle_x_ready:
    CALL __gfx_circle_points
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_R,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JZ circle_done
    MOV16 GFX_I,W
    MOV16 W,0xB0
    INC 0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_I
    JMP circle_y_loop
circle_done:
    RET
.endproc

; fillcircle: same parameters as circle. Circle must fit on screen.
.proc fillcircle
    MOV16 GFX_COLOR,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    CALL gfx_set_color
    MOV16 GFX_R,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 0xB8,W
    MUL 0xB8,W
    MOV16 W,0xB8
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP1
    LDI 0
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_I
fillcircle_y_loop:
fillcircle_find_x:
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 0xB8,W
    MUL 0xB8,W
    MOV16 W,0xB8
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 0xBA,W
    MUL 0xBA,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    ADD 0xBA,W
    MOV16 W,0xB8
    MOV16 GFX_TMP1,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JNC fillcircle_x_ready
    JZ fillcircle_x_ready
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    DEC 0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_TMP0
    JMP fillcircle_find_x
fillcircle_x_ready:
    ; span at cy+y
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB8,W
    SUB 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    ADD 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xBC,W
    ADD 0xB0,W
    MOV16 W,0xBC
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xBC,W
    MOV16 W,0xB4
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    MOV16 0xB4,W
    CALL hline
    ; span at cy-y, skip duplicate y=0
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    LDI 0
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JZ fillcircle_cross_spans
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB8,W
    SUB 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    ADD 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xBC,W
    SUB 0xB0,W
    MOV16 W,0xBC
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xBC,W
    MOV16 W,0xB4
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    MOV16 0xB4,W
    CALL hline
fillcircle_cross_spans:
    ; If x != y, draw the two transposed spans too.
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JZ fillcircle_next_y
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB8,W
    SUB 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    ADD 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xBC,W
    ADD 0xB0,W
    MOV16 W,0xBC
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xBC,W
    MOV16 W,0xB4
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    MOV16 0xB4,W
    CALL hline
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    LDI 0
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JZ fillcircle_next_y
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB8,W
    SUB 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_X0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    ADD 0xB0,W
    MOV16 W,0xBA
    MOV16 GFX_Y0,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBC
    MOV16 GFX_TMP0,W
    MOV16 W,0xB0
    MOV16 0xBC,W
    SUB 0xB0,W
    MOV16 W,0xBC
    MOV16 0xB8,W
    MOV16 W,0xB0
    MOV16 0xBA,W
    MOV16 W,0xB2
    MOV16 0xBC,W
    MOV16 W,0xB4
    MOV16 0xB0,W
    W2F0
    MOV16 0xB2,W
    W2F1
    MOV16 0xB4,W
    CALL hline
fillcircle_next_y:
    MOV16 GFX_I,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xB8
    MOV16 GFX_R,W
    MOV16 W,0xB0
    MOV16 0xB0,W
    MOV16 W,0xBA
    MOV16 0xB8,W
    CMP 0xBA
    JZ fillcircle_done
    MOV16 GFX_I,W
    MOV16 W,0xB0
    INC 0xB0
    MOV16 0xB0,W
    MOV16 W,GFX_I
    JMP fillcircle_y_loop
fillcircle_done:
    RET
.endproc
