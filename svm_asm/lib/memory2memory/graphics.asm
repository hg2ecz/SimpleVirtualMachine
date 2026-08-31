; Memory-to-Memory ISA 320x200x2-bpp graphics helpers.
; Scratch 0x00E8..0x00FA. ABI: A0=color for gfx_set_color/clear;
; gfx_set_palette A0=pointer to four bytes; putpixel A0=x,A1=y;
; hline A0=x0,A1=x1,A2=y; vline A0=x,A1=y0,A2=y1.
.proc gfx_set_color
    STORA [0x00E8], A0
    AND16 [0x00E8], 3
    RET
.endproc

.proc gfx_set_palette
    MOV8 [0xFF0C], [A0+]
    MOV8 [0xFF0D], [A0+]
    MOV8 [0xFF0E], [A0+]
    MOV8 [0xFF0F], [A0+]
    RET
.endproc

.proc putpixel
    STORA [0x00EA], A0
    STORA [0x00EC], A1
    MOV16 [0x00EE], [0x00EC]
    MUL16 [0x00EE], 80
    MOV16 [0x00F0], [0x00EA]
    SHR16 [0x00F0], 2
    ADD16 [0x00EE], [0x00F0]
    MOVA A0, [0x00EE]
    VLD8 [0x00F2], [A0]
    MOV16 [0x00F4], [0x00EA]
    AND16 [0x00F4], 3
    SHL1 [0x00F4]
    MOV16 [0x00F6], 6
    SUB16 [0x00F6], [0x00F4]
    MOV16 [0x00F4], 3
    SHL16 [0x00F4], [0x00F6]
    NOT16 [0x00F4]
    AND16 [0x00F2], [0x00F4]
    MOV16 [0x00F8], [0x00E8]
    SHL16 [0x00F8], [0x00F6]
    OR16 [0x00F2], [0x00F8]
    VST8 [A0], [0x00F2]
    RET
.endproc

.proc clear
    STORA [0x00EA], A0
    AND16 [0x00EA], 3
    MUL16 [0x00EA], 0x55
    MOV16 [0x00EC], 0
    MOV16 [0x00EE], 16000
clear_loop:
    MOVA A0, [0x00EC]
    VST8 [A0], [0x00EA]
    INC16 [0x00EC]
    DEC16 [0x00EE]
    JNZ clear_loop
    RET
.endproc

.proc hline
    STORA [0x00EA], A0
    STORA [0x00EC], A1
    STORA [0x00EE], A2
hline_loop:
    MOVA A0, [0x00EA]
    MOVA A1, [0x00EE]
    CALL putpixel
    CMP16 [0x00EA], [0x00EC]
    JZ hline_done
    INC16 [0x00EA]
    JMP hline_loop
hline_done:
    RET
.endproc

.proc vline
    STORA [0x00EA], A0
    STORA [0x00EC], A1
    STORA [0x00EE], A2
vline_loop:
    MOVA A0, [0x00EA]
    MOVA A1, [0x00EC]
    CALL putpixel
    CMP16 [0x00EC], [0x00EE]
    JZ vline_done
    INC16 [0x00EC]
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
    MOV16 [0x00B0], [GFX_COLOR]
    MOVA A0, [0x00B0]
    CALL gfx_set_color

    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_X1]
    MOV16 [0x00BA], [0x00B0]
    CMP16 [0x00BA], [0x00B8]
    JNC line_x_negative
    MOV16 [0x00B0], [0x00BA]
    SUB16 [0x00B0], [0x00B8]
    MOV16 [GFX_DX], [0x00B0]
    MOV16 [0x00B0], 1
    MOV16 [GFX_SX], [0x00B0]
    JMP line_x_ready
line_x_negative:
    MOV16 [0x00B0], [0x00B8]
    SUB16 [0x00B0], [0x00BA]
    MOV16 [GFX_DX], [0x00B0]
    MOV16 [0x00B0], 0
    MOV16 [GFX_SX], [0x00B0]
line_x_ready:
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_Y1]
    MOV16 [0x00BA], [0x00B0]
    CMP16 [0x00BA], [0x00B8]
    JNC line_y_negative
    MOV16 [0x00B0], [0x00BA]
    SUB16 [0x00B0], [0x00B8]
    MOV16 [GFX_DY], [0x00B0]
    MOV16 [0x00B0], 1
    MOV16 [GFX_SY], [0x00B0]
    JMP line_y_ready
line_y_negative:
    MOV16 [0x00B0], [0x00B8]
    SUB16 [0x00B0], [0x00BA]
    MOV16 [GFX_DY], [0x00B0]
    MOV16 [0x00B0], 0
    MOV16 [GFX_SY], [0x00B0]
line_y_ready:
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [GFX_TMP0], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [GFX_TMP1], [0x00B0]
    MOV16 [0x00B0], 0
    MOV16 [GFX_I], [0x00B0]

    MOV16 [0x00B0], [GFX_DX]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_DY]
    MOV16 [0x00BA], [0x00B0]
    CMP16 [0x00B8], [0x00BA]
    JNC line_vertical_major

line_horizontal_loop:
    ; y = y0 +/- (i*dy/dx), x is maintained in TMP0.
    MOV16 [0x00B0], [GFX_I]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_DY]
    MOV16 [0x00BC], [0x00B0]
    MUL16 [0x00BA], [0x00BC]
    ; dx is non-zero here unless this is a single pixel.
    ; Handle dx=0 explicitly to avoid a DIV-by-zero trap.
    MOV16 [0x00B0], [GFX_DX]
    MOV16 [0x00BC], [0x00B0]
    MOV16 [0x00BE], 0
    CMP16 [0x00BC], [0x00BE]
    JZ line_single_pixel
    DIV16 [0x00BA], [0x00BC]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00B2], [0x00B0]
    MOV16 [0x00B0], [GFX_SY]
    MOV16 [0x00BC], [0x00B0]
    MOV16 [0x00BE], 0
    CMP16 [0x00BC], [0x00BE]
    JZ line_h_sub_y
    ADD16 [0x00B2], [0x00BA]
    JMP line_h_y_ready
line_h_sub_y:
    SUB16 [0x00B2], [0x00BA]
line_h_y_ready:
    MOV16 [0x00B0], [GFX_TMP0]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    CALL putpixel

    MOV16 [0x00B0], [GFX_I]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_DX]
    MOV16 [0x00BA], [0x00B0]
    CMP16 [0x00B8], [0x00BA]
    JZ line_done
    MOV16 [0x00B0], [GFX_SX]
    MOV16 [0x00BC], [0x00B0]
    MOV16 [0x00BE], 0
    CMP16 [0x00BC], [0x00BE]
    MOV16 [0x00B0], [GFX_TMP0]
    JZ line_h_dec_x
    INC16 [0x00B0]
    JMP line_h_store_x
line_h_dec_x:
    DEC16 [0x00B0]
line_h_store_x:
    MOV16 [GFX_TMP0], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    INC16 [0x00B0]
    MOV16 [GFX_I], [0x00B0]
    JMP line_horizontal_loop

line_vertical_major:
    MOV16 [0x00B0], [GFX_DY]
    MOV16 [0x00BC], [0x00B0]
    MOV16 [0x00BE], 0
    CMP16 [0x00BC], [0x00BE]
    JZ line_single_pixel
line_vertical_loop:
    ; x = x0 +/- (i*dx/dy), y is maintained in TMP1.
    MOV16 [0x00B0], [GFX_I]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_DX]
    MOV16 [0x00BC], [0x00B0]
    MUL16 [0x00BA], [0x00BC]
    MOV16 [0x00B0], [GFX_DY]
    MOV16 [0x00BC], [0x00B0]
    DIV16 [0x00BA], [0x00BC]
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_SX]
    MOV16 [0x00BC], [0x00B0]
    MOV16 [0x00BE], 0
    CMP16 [0x00BC], [0x00BE]
    JZ line_v_sub_x
    ADD16 [0x00B8], [0x00BA]
    JMP line_v_x_ready
line_v_sub_x:
    SUB16 [0x00B8], [0x00BA]
line_v_x_ready:
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B0], [GFX_TMP1]
    MOV16 [0x00B2], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    CALL putpixel

    MOV16 [0x00B0], [GFX_I]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_DY]
    MOV16 [0x00BA], [0x00B0]
    CMP16 [0x00B8], [0x00BA]
    JZ line_done
    MOV16 [0x00B0], [GFX_SY]
    MOV16 [0x00BC], [0x00B0]
    MOV16 [0x00BE], 0
    CMP16 [0x00BC], [0x00BE]
    MOV16 [0x00B0], [GFX_TMP1]
    JZ line_v_dec_y
    INC16 [0x00B0]
    JMP line_v_store_y
line_v_dec_y:
    DEC16 [0x00B0]
line_v_store_y:
    MOV16 [GFX_TMP1], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    INC16 [0x00B0]
    MOV16 [GFX_I], [0x00B0]
    JMP line_vertical_loop

line_single_pixel:
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00B2], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    CALL putpixel
line_done:
    RET
.endproc

; rect: GFX_X0=x, GFX_Y0=y, GFX_W=w, GFX_H=h, GFX_COLOR=color.
.proc rect
    MOV16 [0x00B0], [GFX_COLOR]
    MOVA A0, [0x00B0]
    CALL gfx_set_color
    MOV16 [0x00B0], [GFX_W]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00BA], 0
    CMP16 [0x00B8], [0x00BA]
    JZ rect_done
    MOV16 [0x00B0], [GFX_H]
    MOV16 [0x00B8], [0x00B0]
    CMP16 [0x00B8], [0x00BA]
    JZ rect_done

    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_W]
    MOV16 [0x00BA], [0x00B0]
    DEC16 [0x00BA]
    ADD16 [0x00BA], [0x00B8]
    MOV16 [0x00B0], [0x00BA]
    MOV16 [GFX_TMP0], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_H]
    MOV16 [0x00BA], [0x00B0]
    DEC16 [0x00BA]
    ADD16 [0x00BA], [0x00B8]
    MOV16 [0x00B0], [0x00BA]
    MOV16 [GFX_TMP1], [0x00B0]

    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    MOV16 [0x00B2], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00B4], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    MOVA A2, [0x00B4]
    CALL hline

    MOV16 [0x00B0], [GFX_H]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00BA], 1
    CMP16 [0x00B8], [0x00BA]
    JZ rect_sides
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    MOV16 [0x00B2], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP1]
    MOV16 [0x00B4], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    MOVA A2, [0x00B4]
    CALL hline
rect_sides:
    MOV16 [0x00B0], [GFX_H]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00BA], 2
    CMP16 [0x00B8], [0x00BA]
    JNC rect_done
    JZ rect_done
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    INC16 [0x00B0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP1]
    DEC16 [0x00B0]
    MOV16 [0x00B4], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    MOVA A2, [0x00B4]
    CALL vline
    MOV16 [0x00B0], [GFX_W]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00BA], 1
    CMP16 [0x00B8], [0x00BA]
    JZ rect_done
    MOV16 [0x00B0], [GFX_TMP0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    INC16 [0x00B0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP1]
    DEC16 [0x00B0]
    MOV16 [0x00B4], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    MOVA A2, [0x00B4]
    CALL vline
rect_done:
    RET
.endproc

; fillrect: same parameter block as rect.
.proc fillrect
    MOV16 [0x00B0], [GFX_COLOR]
    MOVA A0, [0x00B0]
    CALL gfx_set_color
    MOV16 [0x00B0], [GFX_W]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00BA], 0
    CMP16 [0x00B8], [0x00BA]
    JZ fillrect_done
    MOV16 [0x00B0], [GFX_H]
    MOV16 [0x00B8], [0x00B0]
    CMP16 [0x00B8], [0x00BA]
    JZ fillrect_done
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_W]
    MOV16 [0x00BA], [0x00B0]
    DEC16 [0x00BA]
    ADD16 [0x00BA], [0x00B8]
    MOV16 [0x00B0], [0x00BA]
    MOV16 [GFX_TMP0], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [GFX_TMP1], [0x00B0]
    MOV16 [0x00B0], [GFX_H]
    MOV16 [GFX_TMP2], [0x00B0]
fillrect_loop:
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    MOV16 [0x00B2], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP1]
    MOV16 [0x00B4], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    MOVA A2, [0x00B4]
    CALL hline
    MOV16 [0x00B0], [GFX_TMP2]
    DEC16 [0x00B0]
    MOV16 [GFX_TMP2], [0x00B0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00BA], 0
    CMP16 [0x00B8], [0x00BA]
    JZ fillrect_done
    MOV16 [0x00B0], [GFX_TMP1]
    INC16 [0x00B0]
    MOV16 [GFX_TMP1], [0x00B0]
    JMP fillrect_loop
fillrect_done:
    RET
.endproc

; Internal helper: plot eight symmetric points. Requires TMP0=x, I=y,
; X0=cx, Y0=cy and current color already selected. Circle must fit on screen.
.proc __gfx_circle_points
    ; (cx+x, cy+y)
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    ADD16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    ADD16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    CALL putpixel
    ; (cx+y, cy+x)
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    ADD16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    ADD16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    CALL putpixel
    ; (cx-y, cy+x)
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    SUB16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    ADD16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    CALL putpixel
    ; (cx-x, cy+y)
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    SUB16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    ADD16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    CALL putpixel
    ; (cx-x, cy-y)
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    SUB16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    SUB16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    CALL putpixel
    ; (cx-y, cy-x)
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    SUB16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    SUB16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    CALL putpixel
    ; (cx+y, cy-x)
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    ADD16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    SUB16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    CALL putpixel
    ; (cx+x, cy-y)
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    ADD16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    SUB16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    CALL putpixel
    RET
.endproc

; circle: GFX_X0=cx, GFX_Y0=cy, GFX_R=r, GFX_COLOR=color.
; Uses an integer scan of the first octant; circle must fit on screen.
.proc circle
    MOV16 [0x00B0], [GFX_COLOR]
    MOVA A0, [0x00B0]
    CALL gfx_set_color
    MOV16 [0x00B0], [GFX_R]
    MOV16 [GFX_TMP0], [0x00B0]
    MOV16 [0x00B8], [0x00B0]
    MUL16 [0x00B8], [0x00B8]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [GFX_TMP1], [0x00B0]
    MOV16 [0x00B0], 0
    MOV16 [GFX_I], [0x00B0]
circle_y_loop:
circle_find_x:
    MOV16 [0x00B0], [GFX_TMP0]
    MOV16 [0x00B8], [0x00B0]
    MUL16 [0x00B8], [0x00B8]
    MOV16 [0x00B0], [GFX_I]
    MOV16 [0x00BA], [0x00B0]
    MUL16 [0x00BA], [0x00BA]
    ADD16 [0x00B8], [0x00BA]
    MOV16 [0x00B0], [GFX_TMP1]
    MOV16 [0x00BA], [0x00B0]
    CMP16 [0x00B8], [0x00BA]
    JNC circle_x_ready
    JZ circle_x_ready
    MOV16 [0x00B0], [GFX_TMP0]
    DEC16 [0x00B0]
    MOV16 [GFX_TMP0], [0x00B0]
    JMP circle_find_x
circle_x_ready:
    CALL __gfx_circle_points
    MOV16 [0x00B0], [GFX_I]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_R]
    MOV16 [0x00BA], [0x00B0]
    CMP16 [0x00B8], [0x00BA]
    JZ circle_done
    MOV16 [0x00B0], [GFX_I]
    INC16 [0x00B0]
    MOV16 [GFX_I], [0x00B0]
    JMP circle_y_loop
circle_done:
    RET
.endproc

; fillcircle: same parameters as circle. Circle must fit on screen.
.proc fillcircle
    MOV16 [0x00B0], [GFX_COLOR]
    MOVA A0, [0x00B0]
    CALL gfx_set_color
    MOV16 [0x00B0], [GFX_R]
    MOV16 [GFX_TMP0], [0x00B0]
    MOV16 [0x00B8], [0x00B0]
    MUL16 [0x00B8], [0x00B8]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [GFX_TMP1], [0x00B0]
    MOV16 [0x00B0], 0
    MOV16 [GFX_I], [0x00B0]
fillcircle_y_loop:
fillcircle_find_x:
    MOV16 [0x00B0], [GFX_TMP0]
    MOV16 [0x00B8], [0x00B0]
    MUL16 [0x00B8], [0x00B8]
    MOV16 [0x00B0], [GFX_I]
    MOV16 [0x00BA], [0x00B0]
    MUL16 [0x00BA], [0x00BA]
    ADD16 [0x00B8], [0x00BA]
    MOV16 [0x00B0], [GFX_TMP1]
    MOV16 [0x00BA], [0x00B0]
    CMP16 [0x00B8], [0x00BA]
    JNC fillcircle_x_ready
    JZ fillcircle_x_ready
    MOV16 [0x00B0], [GFX_TMP0]
    DEC16 [0x00B0]
    MOV16 [GFX_TMP0], [0x00B0]
    JMP fillcircle_find_x
fillcircle_x_ready:
    ; span at cy+y
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    SUB16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    ADD16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00BC], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    ADD16 [0x00BC], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOV16 [0x00B4], [0x00BC]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    MOVA A2, [0x00B4]
    CALL hline
    ; span at cy-y, skip duplicate y=0
    MOV16 [0x00B0], [GFX_I]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00BA], 0
    CMP16 [0x00B8], [0x00BA]
    JZ fillcircle_cross_spans
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    SUB16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    ADD16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00BC], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    SUB16 [0x00BC], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOV16 [0x00B4], [0x00BC]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    MOVA A2, [0x00B4]
    CALL hline
fillcircle_cross_spans:
    ; If x != y, draw the two transposed spans too.
    MOV16 [0x00B0], [GFX_TMP0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    MOV16 [0x00BA], [0x00B0]
    CMP16 [0x00B8], [0x00BA]
    JZ fillcircle_next_y
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    SUB16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    ADD16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00BC], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    ADD16 [0x00BC], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOV16 [0x00B4], [0x00BC]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    MOVA A2, [0x00B4]
    CALL hline
    MOV16 [0x00B0], [GFX_TMP0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00BA], 0
    CMP16 [0x00B8], [0x00BA]
    JZ fillcircle_next_y
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    SUB16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_X0]
    MOV16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_I]
    ADD16 [0x00BA], [0x00B0]
    MOV16 [0x00B0], [GFX_Y0]
    MOV16 [0x00BC], [0x00B0]
    MOV16 [0x00B0], [GFX_TMP0]
    SUB16 [0x00BC], [0x00B0]
    MOV16 [0x00B0], [0x00B8]
    MOV16 [0x00B2], [0x00BA]
    MOV16 [0x00B4], [0x00BC]
    MOVA A0, [0x00B0]
    MOVA A1, [0x00B2]
    MOVA A2, [0x00B4]
    CALL hline
fillcircle_next_y:
    MOV16 [0x00B0], [GFX_I]
    MOV16 [0x00B8], [0x00B0]
    MOV16 [0x00B0], [GFX_R]
    MOV16 [0x00BA], [0x00B0]
    CMP16 [0x00B8], [0x00BA]
    JZ fillcircle_done
    MOV16 [0x00B0], [GFX_I]
    INC16 [0x00B0]
    MOV16 [GFX_I], [0x00B0]
    JMP fillcircle_y_loop
fillcircle_done:
    RET
.endproc
