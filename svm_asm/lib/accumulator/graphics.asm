; Accumulator ISA 320x200x2-bpp graphics helpers.
; Scratch 0x00E8 color, 0x00EA..0x00FA temporaries.
; ABI: gfx_set_color A=color; gfx_set_palette X=pointer to four palette bytes;
; putpixel X=x,Y=y; clear A=color; hline X=x0,Y=x1,A=y; vline A=x,X=y0,Y=y1.
.proc gfx_set_color
    ANDI 3
    STA8 0x00E8
    RET
.endproc

.proc gfx_set_palette
    LDA8 [X+]
    STA8 0xFF0C
    LDA8 [X+]
    STA8 0xFF0D
    LDA8 [X+]
    STA8 0xFF0E
    LDA8 [X+]
    STA8 0xFF0F
    RET
.endproc

.proc putpixel
    TXA
    STA16 0x00EA
    TYA
    STA16 0x00EC
    LDXI 80
    MULX
    STA16 0x00EE
    LDA16 0x00EA
    SHR1
    SHR1
    TAX
    LDA16 0x00EE
    ADDX
    STA16 0x00F0
    TAX
    VLDA8 [X]
    STA16 0x00F2
    LDA16 0x00EA
    ANDI 3
    SHL1
    TAX
    LDAI 6
    SUBX
    STA16 0x00F4
    TAX
    LDAI 3
    SHLX
    STA16 0x00F6
    NOT
    TAX
    LDA16 0x00F2
    ANDX
    STA16 0x00F8
    LDA16 0x00F4
    TAX
    LDA8 0x00E8
    SHLX
    TAX
    LDA16 0x00F8
    ORX
    STA16 0x00FA
    LDA16 0x00F0
    TAX
    LDA16 0x00FA
    VSTA8 [X]
    RET
.endproc

.proc clear
    ANDI 3
    LDXI 0x55
    MULX
    LDXI 0
    LDYI 16000
clear_loop:
    VSTA8 [X+]
    DEY
    JNZ clear_loop
    RET
.endproc

.proc hline
    STA16 0x00EE
    TXA
    STA16 0x00EA
    TYA
    STA16 0x00EC
hline_loop:
    LDA16 0x00EA
    TAX
    LDA16 0x00EE
    TAY
    CALL putpixel
    LDA16 0x00EA
    TAX
    LDA16 0x00EC
    CMPX
    JZ hline_done
    LDA16 0x00EA
    INC
    STA16 0x00EA
    JMP hline_loop
hline_done:
    RET
.endproc

.proc vline
    STA16 0x00EA
    TXA
    STA16 0x00EC
    TYA
    STA16 0x00EE
vline_loop:
    LDA16 0x00EA
    TAX
    LDA16 0x00EC
    TAY
    CALL putpixel
    LDA16 0x00EC
    TAX
    LDA16 0x00EE
    CMPX
    JZ vline_done
    LDA16 0x00EC
    INC
    STA16 0x00EC
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
    LDA16 GFX_COLOR
    STA16 0x00B0
    LDA16 0x00B0
    CALL gfx_set_color

    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_X1
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00B8
    TAX
    LDA16 0x00BA
    CMPX
    JNC line_x_negative
    LDA16 0x00BA
    STA16 0x00B0
    LDA16 0x00B8
    TAX
    LDA16 0x00B0
    SUBX
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_DX
    LDAI 1
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_SX
    JMP line_x_ready
line_x_negative:
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    TAX
    LDA16 0x00B0
    SUBX
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_DX
    LDAI 0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_SX
line_x_ready:
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_Y1
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00B8
    TAX
    LDA16 0x00BA
    CMPX
    JNC line_y_negative
    LDA16 0x00BA
    STA16 0x00B0
    LDA16 0x00B8
    TAX
    LDA16 0x00B0
    SUBX
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_DY
    LDAI 1
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_SY
    JMP line_y_ready
line_y_negative:
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    TAX
    LDA16 0x00B0
    SUBX
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_DY
    LDAI 0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_SY
line_y_ready:
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP0
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP1
    LDAI 0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_I

    LDA16 GFX_DX
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_DY
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JNC line_vertical_major

line_horizontal_loop:
    ; y = y0 +/- (i*dy/dx), x is maintained in TMP0.
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_DY
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDA16 0x00BC
    TAX
    LDA16 0x00BA
    MULX
    STA16 0x00BA
    ; dx is non-zero here unless this is a single pixel.
    ; Handle dx=0 explicitly to avoid a DIV-by-zero trap.
    LDA16 GFX_DX
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDAI 0
    STA16 0x00BE
    LDA16 0x00BE
    TAX
    LDA16 0x00BC
    CMPX
    JZ line_single_pixel
    LDA16 0x00BC
    TAX
    LDA16 0x00BA
    DIVX
    STA16 0x00BA
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B2
    LDA16 GFX_SY
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDAI 0
    STA16 0x00BE
    LDA16 0x00BE
    TAX
    LDA16 0x00BC
    CMPX
    JZ line_h_sub_y
    LDA16 0x00BA
    TAX
    LDA16 0x00B2
    ADDX
    STA16 0x00B2
    JMP line_h_y_ready
line_h_sub_y:
    LDA16 0x00BA
    TAX
    LDA16 0x00B2
    SUBX
    STA16 0x00B2
line_h_y_ready:
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    CALL putpixel

    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_DX
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ line_done
    LDA16 GFX_SX
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDAI 0
    STA16 0x00BE
    LDA16 0x00BE
    TAX
    LDA16 0x00BC
    CMPX
    LDA16 GFX_TMP0
    STA16 0x00B0
    JZ line_h_dec_x
    LDA16 0x00B0
    INC
    STA16 0x00B0
    JMP line_h_store_x
line_h_dec_x:
    LDA16 0x00B0
    DEC
    STA16 0x00B0
line_h_store_x:
    LDA16 0x00B0
    STA16 GFX_TMP0
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    INC
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_I
    JMP line_horizontal_loop

line_vertical_major:
    LDA16 GFX_DY
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDAI 0
    STA16 0x00BE
    LDA16 0x00BE
    TAX
    LDA16 0x00BC
    CMPX
    JZ line_single_pixel
line_vertical_loop:
    ; x = x0 +/- (i*dx/dy), y is maintained in TMP1.
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_DX
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDA16 0x00BC
    TAX
    LDA16 0x00BA
    MULX
    STA16 0x00BA
    LDA16 GFX_DY
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDA16 0x00BC
    TAX
    LDA16 0x00BA
    DIVX
    STA16 0x00BA
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_SX
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDAI 0
    STA16 0x00BE
    LDA16 0x00BE
    TAX
    LDA16 0x00BC
    CMPX
    JZ line_v_sub_x
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    ADDX
    STA16 0x00B8
    JMP line_v_x_ready
line_v_sub_x:
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    SUBX
    STA16 0x00B8
line_v_x_ready:
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 GFX_TMP1
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B2
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    CALL putpixel

    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_DY
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ line_done
    LDA16 GFX_SY
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDAI 0
    STA16 0x00BE
    LDA16 0x00BE
    TAX
    LDA16 0x00BC
    CMPX
    LDA16 GFX_TMP1
    STA16 0x00B0
    JZ line_v_dec_y
    LDA16 0x00B0
    INC
    STA16 0x00B0
    JMP line_v_store_y
line_v_dec_y:
    LDA16 0x00B0
    DEC
    STA16 0x00B0
line_v_store_y:
    LDA16 0x00B0
    STA16 GFX_TMP1
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    INC
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_I
    JMP line_vertical_loop

line_single_pixel:
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B2
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    CALL putpixel
line_done:
    RET
.endproc

; rect: GFX_X0=x, GFX_Y0=y, GFX_W=w, GFX_H=h, GFX_COLOR=color.
.proc rect
    LDA16 GFX_COLOR
    STA16 0x00B0
    LDA16 0x00B0
    CALL gfx_set_color
    LDA16 GFX_W
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDAI 0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ rect_done
    LDA16 GFX_H
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ rect_done

    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_W
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    DEC
    STA16 0x00BA
    LDA16 0x00B8
    TAX
    LDA16 0x00BA
    ADDX
    STA16 0x00BA
    LDA16 0x00BA
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP0
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_H
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    DEC
    STA16 0x00BA
    LDA16 0x00B8
    TAX
    LDA16 0x00BA
    ADDX
    STA16 0x00BA
    LDA16 0x00BA
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP1

    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B2
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B4
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    LDA16 0x00B4
    CALL hline

    LDA16 GFX_H
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDAI 1
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ rect_sides
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B2
    LDA16 GFX_TMP1
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B4
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    LDA16 0x00B4
    CALL hline
rect_sides:
    LDA16 GFX_H
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDAI 2
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JNC rect_done
    JZ rect_done
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    INC
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_TMP1
    STA16 0x00B0
    LDA16 0x00B0
    DEC
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B4
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00B2
    TAX
    LDA16 0x00B4
    TAY
    LDA16 0x00B0
    CALL vline
    LDA16 GFX_W
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDAI 1
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ rect_done
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    INC
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_TMP1
    STA16 0x00B0
    LDA16 0x00B0
    DEC
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B4
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00B2
    TAX
    LDA16 0x00B4
    TAY
    LDA16 0x00B0
    CALL vline
rect_done:
    RET
.endproc

; fillrect: same parameter block as rect.
.proc fillrect
    LDA16 GFX_COLOR
    STA16 0x00B0
    LDA16 0x00B0
    CALL gfx_set_color
    LDA16 GFX_W
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDAI 0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ fillrect_done
    LDA16 GFX_H
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ fillrect_done
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_W
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    DEC
    STA16 0x00BA
    LDA16 0x00B8
    TAX
    LDA16 0x00BA
    ADDX
    STA16 0x00BA
    LDA16 0x00BA
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP0
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP1
    LDA16 GFX_H
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP2
fillrect_loop:
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B2
    LDA16 GFX_TMP1
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B4
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    LDA16 0x00B4
    CALL hline
    LDA16 GFX_TMP2
    STA16 0x00B0
    LDA16 0x00B0
    DEC
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP2
    LDA16 0x00B0
    STA16 0x00B8
    LDAI 0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ fillrect_done
    LDA16 GFX_TMP1
    STA16 0x00B0
    LDA16 0x00B0
    INC
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP1
    JMP fillrect_loop
fillrect_done:
    RET
.endproc

; Internal helper: plot eight symmetric points. Requires TMP0=x, I=y,
; X0=cx, Y0=cy and current color already selected. Circle must fit on screen.
.proc __gfx_circle_points
    ; (cx+x, cy+y)
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B8
    ADDX
    STA16 0x00B8
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BA
    ADDX
    STA16 0x00BA
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    CALL putpixel
    ; (cx+y, cy+x)
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B8
    ADDX
    STA16 0x00B8
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BA
    ADDX
    STA16 0x00BA
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    CALL putpixel
    ; (cx-y, cy+x)
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B8
    SUBX
    STA16 0x00B8
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BA
    ADDX
    STA16 0x00BA
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    CALL putpixel
    ; (cx-x, cy+y)
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B8
    SUBX
    STA16 0x00B8
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BA
    ADDX
    STA16 0x00BA
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    CALL putpixel
    ; (cx-x, cy-y)
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B8
    SUBX
    STA16 0x00B8
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BA
    SUBX
    STA16 0x00BA
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    CALL putpixel
    ; (cx-y, cy-x)
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B8
    SUBX
    STA16 0x00B8
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BA
    SUBX
    STA16 0x00BA
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    CALL putpixel
    ; (cx+y, cy-x)
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B8
    ADDX
    STA16 0x00B8
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BA
    SUBX
    STA16 0x00BA
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    CALL putpixel
    ; (cx+x, cy-y)
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B8
    ADDX
    STA16 0x00B8
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BA
    SUBX
    STA16 0x00BA
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    CALL putpixel
    RET
.endproc

; circle: GFX_X0=cx, GFX_Y0=cy, GFX_R=r, GFX_COLOR=color.
; Uses an integer scan of the first octant; circle must fit on screen.
.proc circle
    LDA16 GFX_COLOR
    STA16 0x00B0
    LDA16 0x00B0
    CALL gfx_set_color
    LDA16 GFX_R
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 0x00B8
    TAX
    LDA16 0x00B8
    MULX
    STA16 0x00B8
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP1
    LDAI 0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_I
circle_y_loop:
circle_find_x:
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 0x00B8
    TAX
    LDA16 0x00B8
    MULX
    STA16 0x00B8
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00BA
    MULX
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    ADDX
    STA16 0x00B8
    LDA16 GFX_TMP1
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JNC circle_x_ready
    JZ circle_x_ready
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    DEC
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP0
    JMP circle_find_x
circle_x_ready:
    CALL __gfx_circle_points
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_R
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ circle_done
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    INC
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_I
    JMP circle_y_loop
circle_done:
    RET
.endproc

; fillcircle: same parameters as circle. Circle must fit on screen.
.proc fillcircle
    LDA16 GFX_COLOR
    STA16 0x00B0
    LDA16 0x00B0
    CALL gfx_set_color
    LDA16 GFX_R
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 0x00B8
    TAX
    LDA16 0x00B8
    MULX
    STA16 0x00B8
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP1
    LDAI 0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_I
fillcircle_y_loop:
fillcircle_find_x:
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 0x00B8
    TAX
    LDA16 0x00B8
    MULX
    STA16 0x00B8
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00BA
    MULX
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    ADDX
    STA16 0x00B8
    LDA16 GFX_TMP1
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JNC fillcircle_x_ready
    JZ fillcircle_x_ready
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    DEC
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_TMP0
    JMP fillcircle_find_x
fillcircle_x_ready:
    ; span at cy+y
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B8
    SUBX
    STA16 0x00B8
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BA
    ADDX
    STA16 0x00BA
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BC
    ADDX
    STA16 0x00BC
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00BC
    STA16 0x00B4
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    LDA16 0x00B4
    CALL hline
    ; span at cy-y, skip duplicate y=0
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDAI 0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ fillcircle_cross_spans
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B8
    SUBX
    STA16 0x00B8
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BA
    ADDX
    STA16 0x00BA
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BC
    SUBX
    STA16 0x00BC
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00BC
    STA16 0x00B4
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    LDA16 0x00B4
    CALL hline
fillcircle_cross_spans:
    ; If x != y, draw the two transposed spans too.
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ fillcircle_next_y
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B8
    SUBX
    STA16 0x00B8
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BA
    ADDX
    STA16 0x00BA
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BC
    ADDX
    STA16 0x00BC
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00BC
    STA16 0x00B4
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    LDA16 0x00B4
    CALL hline
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDAI 0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ fillcircle_next_y
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00B8
    SUBX
    STA16 0x00B8
    LDA16 GFX_X0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BA
    ADDX
    STA16 0x00BA
    LDA16 GFX_Y0
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BC
    LDA16 GFX_TMP0
    STA16 0x00B0
    LDA16 0x00B0
    TAX
    LDA16 0x00BC
    SUBX
    STA16 0x00BC
    LDA16 0x00B8
    STA16 0x00B0
    LDA16 0x00BA
    STA16 0x00B2
    LDA16 0x00BC
    STA16 0x00B4
    LDA16 0x00B0
    TAX
    LDA16 0x00B2
    TAY
    LDA16 0x00B4
    CALL hline
fillcircle_next_y:
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00B8
    LDA16 GFX_R
    STA16 0x00B0
    LDA16 0x00B0
    STA16 0x00BA
    LDA16 0x00BA
    TAX
    LDA16 0x00B8
    CMPX
    JZ fillcircle_done
    LDA16 GFX_I
    STA16 0x00B0
    LDA16 0x00B0
    INC
    STA16 0x00B0
    LDA16 0x00B0
    STA16 GFX_I
    JMP fillcircle_y_loop
fillcircle_done:
    RET
.endproc
