; Load/Store ISA 320x200x2-bpp graphics helpers.
; Scratch: 0x00E8 current color. ABI mirrors Register ISA.
; gfx_set_color R0=color; gfx_set_palette R0..R3; putpixel R0=x,R1=y;
; clear R0=color; hline R0=x0,R1=x1,R2=y; vline R0=x,R1=y0,R2=y1.
.proc gfx_set_color
    ANDI R0, 3
    LDI R1, 0x00E8
    STORE8 [R1], R0
    RET
.endproc

.proc gfx_set_palette
    LDI R4, 0xFF0C
    STORE8 [R4], R0
    INC R4
    STORE8 [R4], R1
    INC R4
    STORE8 [R4], R2
    INC R4
    STORE8 [R4], R3
    RET
.endproc

.proc putpixel
    MOV R2, R1
    LDI R3, 80
    MUL R2, R3
    MOV R4, R0
    LDI R5, 2
    SHR R4, R5
    ADD R2, R4
    MOV R4, R0
    ANDI R4, 3
    SHL1 R4
    LDI R5, 6
    SUB R5, R4
    LDI R3, 3
    SHL R3, R5
    VLOAD8 R4, [R2]
    MOV R7, R3
    NOT R7
    AND R4, R7
    LDI R6, 0x00E8
    LOAD8 R6, [R6]
    SHL R6, R5
    OR R4, R6
    VSTORE8 [R2], R4
    RET
.endproc

.proc clear
    ANDI R0, 3
    LDI R1, 0x55
    MUL R0, R1
    LDI R1, 0
    LDI R2, 16000
clear_loop:
    VSTORE8 [R1], R0
    INC R1
    DEC R2
    JNZ clear_loop
    RET
.endproc

.proc hline
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
.endproc

.proc vline
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
    ZLOAD16 GFX_COLOR
    CALL gfx_set_color

    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_X1
    MOV R5, R0
    CMP R5, R4
    JNC line_x_negative
    MOV R0, R5
    SUB R0, R4
    ZSTORE16 GFX_DX
    MOVI R0, 1
    ZSTORE16 GFX_SX
    JMP line_x_ready
line_x_negative:
    MOV R0, R4
    SUB R0, R5
    ZSTORE16 GFX_DX
    MOVI R0, 0
    ZSTORE16 GFX_SX
line_x_ready:
    ZLOAD16 GFX_Y0
    MOV R4, R0
    ZLOAD16 GFX_Y1
    MOV R5, R0
    CMP R5, R4
    JNC line_y_negative
    MOV R0, R5
    SUB R0, R4
    ZSTORE16 GFX_DY
    MOVI R0, 1
    ZSTORE16 GFX_SY
    JMP line_y_ready
line_y_negative:
    MOV R0, R4
    SUB R0, R5
    ZSTORE16 GFX_DY
    MOVI R0, 0
    ZSTORE16 GFX_SY
line_y_ready:
    ZLOAD16 GFX_X0
    ZSTORE16 GFX_TMP0
    ZLOAD16 GFX_Y0
    ZSTORE16 GFX_TMP1
    MOVI R0, 0
    ZSTORE16 GFX_I

    ZLOAD16 GFX_DX
    MOV R4, R0
    ZLOAD16 GFX_DY
    MOV R5, R0
    CMP R4, R5
    JNC line_vertical_major

line_horizontal_loop:
    ; y = y0 +/- (i*dy/dx), x is maintained in TMP0.
    ZLOAD16 GFX_I
    MOV R5, R0
    ZLOAD16 GFX_DY
    MOV R6, R0
    MUL R5, R6
    ; dx is non-zero here unless this is a single pixel.
    ; Handle dx=0 explicitly to avoid a DIV-by-zero trap.
    ZLOAD16 GFX_DX
    MOV R6, R0
    MOVI R7, 0
    CMP R6, R7
    JZ line_single_pixel
    DIV R5, R6
    ZLOAD16 GFX_Y0
    MOV R1, R0
    ZLOAD16 GFX_SY
    MOV R6, R0
    MOVI R7, 0
    CMP R6, R7
    JZ line_h_sub_y
    ADD R1, R5
    JMP line_h_y_ready
line_h_sub_y:
    SUB R1, R5
line_h_y_ready:
    ZLOAD16 GFX_TMP0
    CALL putpixel

    ZLOAD16 GFX_I
    MOV R4, R0
    ZLOAD16 GFX_DX
    MOV R5, R0
    CMP R4, R5
    JZ line_done
    ZLOAD16 GFX_SX
    MOV R6, R0
    MOVI R7, 0
    CMP R6, R7
    ZLOAD16 GFX_TMP0
    JZ line_h_dec_x
    INC R0
    JMP line_h_store_x
line_h_dec_x:
    DEC R0
line_h_store_x:
    ZSTORE16 GFX_TMP0
    ZLOAD16 GFX_I
    INC R0
    ZSTORE16 GFX_I
    JMP line_horizontal_loop

line_vertical_major:
    ZLOAD16 GFX_DY
    MOV R6, R0
    MOVI R7, 0
    CMP R6, R7
    JZ line_single_pixel
line_vertical_loop:
    ; x = x0 +/- (i*dx/dy), y is maintained in TMP1.
    ZLOAD16 GFX_I
    MOV R5, R0
    ZLOAD16 GFX_DX
    MOV R6, R0
    MUL R5, R6
    ZLOAD16 GFX_DY
    MOV R6, R0
    DIV R5, R6
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_SX
    MOV R6, R0
    MOVI R7, 0
    CMP R6, R7
    JZ line_v_sub_x
    ADD R4, R5
    JMP line_v_x_ready
line_v_sub_x:
    SUB R4, R5
line_v_x_ready:
    MOV R0, R4
    ZLOAD16 GFX_TMP1
    MOV R1, R0
    MOV R0, R4
    CALL putpixel

    ZLOAD16 GFX_I
    MOV R4, R0
    ZLOAD16 GFX_DY
    MOV R5, R0
    CMP R4, R5
    JZ line_done
    ZLOAD16 GFX_SY
    MOV R6, R0
    MOVI R7, 0
    CMP R6, R7
    ZLOAD16 GFX_TMP1
    JZ line_v_dec_y
    INC R0
    JMP line_v_store_y
line_v_dec_y:
    DEC R0
line_v_store_y:
    ZSTORE16 GFX_TMP1
    ZLOAD16 GFX_I
    INC R0
    ZSTORE16 GFX_I
    JMP line_vertical_loop

line_single_pixel:
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_Y0
    MOV R1, R0
    MOV R0, R4
    CALL putpixel
line_done:
    RET
.endproc

; rect: GFX_X0=x, GFX_Y0=y, GFX_W=w, GFX_H=h, GFX_COLOR=color.
.proc rect
    ZLOAD16 GFX_COLOR
    CALL gfx_set_color
    ZLOAD16 GFX_W
    MOV R4, R0
    MOVI R5, 0
    CMP R4, R5
    JZ rect_done
    ZLOAD16 GFX_H
    MOV R4, R0
    CMP R4, R5
    JZ rect_done

    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_W
    MOV R5, R0
    DEC R5
    ADD R5, R4
    MOV R0, R5
    ZSTORE16 GFX_TMP0
    ZLOAD16 GFX_Y0
    MOV R4, R0
    ZLOAD16 GFX_H
    MOV R5, R0
    DEC R5
    ADD R5, R4
    MOV R0, R5
    ZSTORE16 GFX_TMP1

    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_TMP0
    MOV R1, R0
    ZLOAD16 GFX_Y0
    MOV R2, R0
    MOV R0, R4
    CALL hline

    ZLOAD16 GFX_H
    MOV R4, R0
    MOVI R5, 1
    CMP R4, R5
    JZ rect_sides
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_TMP0
    MOV R1, R0
    ZLOAD16 GFX_TMP1
    MOV R2, R0
    MOV R0, R4
    CALL hline
rect_sides:
    ZLOAD16 GFX_H
    MOV R4, R0
    MOVI R5, 2
    CMP R4, R5
    JNC rect_done
    JZ rect_done
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_Y0
    INC R0
    MOV R5, R0
    ZLOAD16 GFX_TMP1
    DEC R0
    MOV R2, R0
    MOV R0, R4
    MOV R1, R5
    CALL vline
    ZLOAD16 GFX_W
    MOV R4, R0
    MOVI R5, 1
    CMP R4, R5
    JZ rect_done
    ZLOAD16 GFX_TMP0
    MOV R4, R0
    ZLOAD16 GFX_Y0
    INC R0
    MOV R5, R0
    ZLOAD16 GFX_TMP1
    DEC R0
    MOV R2, R0
    MOV R0, R4
    MOV R1, R5
    CALL vline
rect_done:
    RET
.endproc

; fillrect: same parameter block as rect.
.proc fillrect
    ZLOAD16 GFX_COLOR
    CALL gfx_set_color
    ZLOAD16 GFX_W
    MOV R4, R0
    MOVI R5, 0
    CMP R4, R5
    JZ fillrect_done
    ZLOAD16 GFX_H
    MOV R4, R0
    CMP R4, R5
    JZ fillrect_done
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_W
    MOV R5, R0
    DEC R5
    ADD R5, R4
    MOV R0, R5
    ZSTORE16 GFX_TMP0
    ZLOAD16 GFX_Y0
    ZSTORE16 GFX_TMP1
    ZLOAD16 GFX_H
    ZSTORE16 GFX_TMP2
fillrect_loop:
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_TMP0
    MOV R1, R0
    ZLOAD16 GFX_TMP1
    MOV R2, R0
    MOV R0, R4
    CALL hline
    ZLOAD16 GFX_TMP2
    DEC R0
    ZSTORE16 GFX_TMP2
    MOV R4, R0
    MOVI R5, 0
    CMP R4, R5
    JZ fillrect_done
    ZLOAD16 GFX_TMP1
    INC R0
    ZSTORE16 GFX_TMP1
    JMP fillrect_loop
fillrect_done:
    RET
.endproc

; Internal helper: plot eight symmetric points. Requires TMP0=x, I=y,
; X0=cx, Y0=cy and current color already selected. Circle must fit on screen.
.proc __gfx_circle_points
    ; (cx+x, cy+y)
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_TMP0
    ADD R4, R0
    ZLOAD16 GFX_Y0
    MOV R5, R0
    ZLOAD16 GFX_I
    ADD R5, R0
    MOV R0, R4
    MOV R1, R5
    CALL putpixel
    ; (cx+y, cy+x)
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_I
    ADD R4, R0
    ZLOAD16 GFX_Y0
    MOV R5, R0
    ZLOAD16 GFX_TMP0
    ADD R5, R0
    MOV R0, R4
    MOV R1, R5
    CALL putpixel
    ; (cx-y, cy+x)
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_I
    SUB R4, R0
    ZLOAD16 GFX_Y0
    MOV R5, R0
    ZLOAD16 GFX_TMP0
    ADD R5, R0
    MOV R0, R4
    MOV R1, R5
    CALL putpixel
    ; (cx-x, cy+y)
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_TMP0
    SUB R4, R0
    ZLOAD16 GFX_Y0
    MOV R5, R0
    ZLOAD16 GFX_I
    ADD R5, R0
    MOV R0, R4
    MOV R1, R5
    CALL putpixel
    ; (cx-x, cy-y)
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_TMP0
    SUB R4, R0
    ZLOAD16 GFX_Y0
    MOV R5, R0
    ZLOAD16 GFX_I
    SUB R5, R0
    MOV R0, R4
    MOV R1, R5
    CALL putpixel
    ; (cx-y, cy-x)
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_I
    SUB R4, R0
    ZLOAD16 GFX_Y0
    MOV R5, R0
    ZLOAD16 GFX_TMP0
    SUB R5, R0
    MOV R0, R4
    MOV R1, R5
    CALL putpixel
    ; (cx+y, cy-x)
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_I
    ADD R4, R0
    ZLOAD16 GFX_Y0
    MOV R5, R0
    ZLOAD16 GFX_TMP0
    SUB R5, R0
    MOV R0, R4
    MOV R1, R5
    CALL putpixel
    ; (cx+x, cy-y)
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_TMP0
    ADD R4, R0
    ZLOAD16 GFX_Y0
    MOV R5, R0
    ZLOAD16 GFX_I
    SUB R5, R0
    MOV R0, R4
    MOV R1, R5
    CALL putpixel
    RET
.endproc

; circle: GFX_X0=cx, GFX_Y0=cy, GFX_R=r, GFX_COLOR=color.
; Uses an integer scan of the first octant; circle must fit on screen.
.proc circle
    ZLOAD16 GFX_COLOR
    CALL gfx_set_color
    ZLOAD16 GFX_R
    ZSTORE16 GFX_TMP0
    MOV R4, R0
    MUL R4, R4
    MOV R0, R4
    ZSTORE16 GFX_TMP1
    MOVI R0, 0
    ZSTORE16 GFX_I
circle_y_loop:
circle_find_x:
    ZLOAD16 GFX_TMP0
    MOV R4, R0
    MUL R4, R4
    ZLOAD16 GFX_I
    MOV R5, R0
    MUL R5, R5
    ADD R4, R5
    ZLOAD16 GFX_TMP1
    MOV R5, R0
    CMP R4, R5
    JNC circle_x_ready
    JZ circle_x_ready
    ZLOAD16 GFX_TMP0
    DEC R0
    ZSTORE16 GFX_TMP0
    JMP circle_find_x
circle_x_ready:
    CALL __gfx_circle_points
    ZLOAD16 GFX_I
    MOV R4, R0
    ZLOAD16 GFX_R
    MOV R5, R0
    CMP R4, R5
    JZ circle_done
    ZLOAD16 GFX_I
    INC R0
    ZSTORE16 GFX_I
    JMP circle_y_loop
circle_done:
    RET
.endproc

; fillcircle: same parameters as circle. Circle must fit on screen.
.proc fillcircle
    ZLOAD16 GFX_COLOR
    CALL gfx_set_color
    ZLOAD16 GFX_R
    ZSTORE16 GFX_TMP0
    MOV R4, R0
    MUL R4, R4
    MOV R0, R4
    ZSTORE16 GFX_TMP1
    MOVI R0, 0
    ZSTORE16 GFX_I
fillcircle_y_loop:
fillcircle_find_x:
    ZLOAD16 GFX_TMP0
    MOV R4, R0
    MUL R4, R4
    ZLOAD16 GFX_I
    MOV R5, R0
    MUL R5, R5
    ADD R4, R5
    ZLOAD16 GFX_TMP1
    MOV R5, R0
    CMP R4, R5
    JNC fillcircle_x_ready
    JZ fillcircle_x_ready
    ZLOAD16 GFX_TMP0
    DEC R0
    ZSTORE16 GFX_TMP0
    JMP fillcircle_find_x
fillcircle_x_ready:
    ; span at cy+y
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_TMP0
    SUB R4, R0
    ZLOAD16 GFX_X0
    MOV R5, R0
    ZLOAD16 GFX_TMP0
    ADD R5, R0
    ZLOAD16 GFX_Y0
    MOV R6, R0
    ZLOAD16 GFX_I
    ADD R6, R0
    MOV R0, R4
    MOV R1, R5
    MOV R2, R6
    CALL hline
    ; span at cy-y, skip duplicate y=0
    ZLOAD16 GFX_I
    MOV R4, R0
    MOVI R5, 0
    CMP R4, R5
    JZ fillcircle_cross_spans
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_TMP0
    SUB R4, R0
    ZLOAD16 GFX_X0
    MOV R5, R0
    ZLOAD16 GFX_TMP0
    ADD R5, R0
    ZLOAD16 GFX_Y0
    MOV R6, R0
    ZLOAD16 GFX_I
    SUB R6, R0
    MOV R0, R4
    MOV R1, R5
    MOV R2, R6
    CALL hline
fillcircle_cross_spans:
    ; If x != y, draw the two transposed spans too.
    ZLOAD16 GFX_TMP0
    MOV R4, R0
    ZLOAD16 GFX_I
    MOV R5, R0
    CMP R4, R5
    JZ fillcircle_next_y
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_I
    SUB R4, R0
    ZLOAD16 GFX_X0
    MOV R5, R0
    ZLOAD16 GFX_I
    ADD R5, R0
    ZLOAD16 GFX_Y0
    MOV R6, R0
    ZLOAD16 GFX_TMP0
    ADD R6, R0
    MOV R0, R4
    MOV R1, R5
    MOV R2, R6
    CALL hline
    ZLOAD16 GFX_TMP0
    MOV R4, R0
    MOVI R5, 0
    CMP R4, R5
    JZ fillcircle_next_y
    ZLOAD16 GFX_X0
    MOV R4, R0
    ZLOAD16 GFX_I
    SUB R4, R0
    ZLOAD16 GFX_X0
    MOV R5, R0
    ZLOAD16 GFX_I
    ADD R5, R0
    ZLOAD16 GFX_Y0
    MOV R6, R0
    ZLOAD16 GFX_TMP0
    SUB R6, R0
    MOV R0, R4
    MOV R1, R5
    MOV R2, R6
    CALL hline
fillcircle_next_y:
    ZLOAD16 GFX_I
    MOV R4, R0
    ZLOAD16 GFX_R
    MOV R5, R0
    CMP R4, R5
    JZ fillcircle_done
    ZLOAD16 GFX_I
    INC R0
    ZSTORE16 GFX_I
    JMP fillcircle_y_loop
fillcircle_done:
    RET
.endproc
