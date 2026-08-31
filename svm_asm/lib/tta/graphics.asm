; TTA16 320x200x2-bpp graphics helpers.
; Scratch byte 0x00E8 is current color. ABI mirrors Register ISA.
.proc gfx_set_color
    MOV R0, ALU.X
    MOV 3, ALU.AND
    MOV 0x00E8, MEM.ADDR
    MOV ALU.OUT, MEM.W8
    RET
.endproc

.proc gfx_set_palette
    MOV 0xFF0C, MEM.ADDR
    MOV R0, MEM.W8
    MOV 0xFF0D, MEM.ADDR
    MOV R1, MEM.W8
    MOV 0xFF0E, MEM.ADDR
    MOV R2, MEM.W8
    MOV 0xFF0F, MEM.ADDR
    MOV R3, MEM.W8
    RET
.endproc

.proc putpixel
    MOV R1, ALU.X
    MOV 80, ALU.MUL
    MOV ALU.OUT, R2
    MOV R0, ALU.X
    MOV 2, ALU.SHR
    MOV ALU.OUT, R3
    MOV R2, ALU.X
    MOV R3, ALU.ADD
    MOV ALU.OUT, R2
    MOV R0, ALU.X
    MOV 3, ALU.AND
    MOV ALU.OUT, R4
    MOV R4, ALU.X
    MOV 1, ALU.SHL
    MOV ALU.OUT, R4
    MOV 6, ALU.X
    MOV R4, ALU.SUB
    MOV ALU.OUT, R5
    MOV 3, ALU.X
    MOV R5, ALU.SHL
    MOV ALU.OUT, R3
    MOV R2, VMEM.ADDR
    MOV VMEM.R8, R4
    MOV R3, ALU.X
    MOV 0, ALU.NOT
    MOV ALU.OUT, R7
    MOV R4, ALU.X
    MOV R7, ALU.AND
    MOV ALU.OUT, R4
    MOV 0x00E8, MEM.ADDR
    MOV MEM.R8, R6
    MOV R6, ALU.X
    MOV R5, ALU.SHL
    MOV ALU.OUT, R6
    MOV R4, ALU.X
    MOV R6, ALU.OR
    MOV ALU.OUT, R4
    MOV R2, VMEM.ADDR
    MOV R4, VMEM.W8
    RET
.endproc

.proc clear
    MOV R0, ALU.X
    MOV 3, ALU.AND
    MOV ALU.OUT, R0
    MOV R0, ALU.X
    MOV 0x55, ALU.MUL
    MOV ALU.OUT, R0
    MOV 0, R1
    MOV 16000, R2
clear_loop:
    MOV R1, VMEM.ADDR
    MOV R0, VMEM.W8
    MOV R1, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R1
    MOV R2, ALU.X
    MOV 1, ALU.SUB
    MOV ALU.OUT, R2
    JNZ clear_loop
    RET
.endproc

.proc hline
    MOV R0, R4
    MOV R1, R5
    MOV R2, R6
hline_loop:
    PUSH R4
    PUSH R5
    PUSH R6
    MOV R4, R0
    MOV R6, R1
    CALL putpixel
    POP R6
    POP R5
    POP R4
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    JZ hline_done
    MOV R4, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R4
    JMP hline_loop
hline_done:
    RET
.endproc

.proc vline
    MOV R0, R4
    MOV R1, R5
    MOV R2, R6
vline_loop:
    PUSH R4
    PUSH R5
    PUSH R6
    MOV R4, R0
    MOV R5, R1
    CALL putpixel
    POP R6
    POP R5
    POP R4
    MOV R5, ALU.X
    MOV R6, ALU.CMP
    JZ vline_done
    MOV R5, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R5
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
    MOV GFX_COLOR, MEM.ADDR
    MOV MEM.R16, R0
    MOV gfx_set_color, CTRL.CALL

    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_X1, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R5, ALU.X
    MOV R4, ALU.CMP
    MOV line_x_negative, CTRL.JNC
    MOV R5, R0
    MOV R0, ALU.X
    MOV R4, ALU.SUB
    MOV ALU.OUT, R0
    MOV GFX_DX, MEM.ADDR
    MOV R0, MEM.W16
    MOV 1, R0
    MOV GFX_SX, MEM.ADDR
    MOV R0, MEM.W16
    MOV line_x_ready, CTRL.JMP
line_x_negative:
    MOV R4, R0
    MOV R0, ALU.X
    MOV R5, ALU.SUB
    MOV ALU.OUT, R0
    MOV GFX_DX, MEM.ADDR
    MOV R0, MEM.W16
    MOV 0, R0
    MOV GFX_SX, MEM.ADDR
    MOV R0, MEM.W16
line_x_ready:
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_Y1, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R5, ALU.X
    MOV R4, ALU.CMP
    MOV line_y_negative, CTRL.JNC
    MOV R5, R0
    MOV R0, ALU.X
    MOV R4, ALU.SUB
    MOV ALU.OUT, R0
    MOV GFX_DY, MEM.ADDR
    MOV R0, MEM.W16
    MOV 1, R0
    MOV GFX_SY, MEM.ADDR
    MOV R0, MEM.W16
    MOV line_y_ready, CTRL.JMP
line_y_negative:
    MOV R4, R0
    MOV R0, ALU.X
    MOV R5, ALU.SUB
    MOV ALU.OUT, R0
    MOV GFX_DY, MEM.ADDR
    MOV R0, MEM.W16
    MOV 0, R0
    MOV GFX_SY, MEM.ADDR
    MOV R0, MEM.W16
line_y_ready:
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV GFX_TMP0, MEM.ADDR
    MOV R0, MEM.W16
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV GFX_TMP1, MEM.ADDR
    MOV R0, MEM.W16
    MOV 0, R0
    MOV GFX_I, MEM.ADDR
    MOV R0, MEM.W16

    MOV GFX_DX, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_DY, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV line_vertical_major, CTRL.JNC

line_horizontal_loop:
    ; y = y0 +/- (i*dy/dx), x is maintained in TMP0.
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_DY, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV R5, ALU.X
    MOV R6, ALU.MUL
    MOV ALU.OUT, R5
    ; dx is non-zero here unless this is a single pixel.
    ; Handle dx=0 explicitly to avoid a DIV-by-zero trap.
    MOV GFX_DX, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV 0, R7
    MOV R6, ALU.X
    MOV R7, ALU.CMP
    MOV line_single_pixel, CTRL.JZ
    MOV R5, ALU.X
    MOV R6, ALU.DIV
    MOV ALU.OUT, R5
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R1
    MOV GFX_SY, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV 0, R7
    MOV R6, ALU.X
    MOV R7, ALU.CMP
    MOV line_h_sub_y, CTRL.JZ
    MOV R1, ALU.X
    MOV R5, ALU.ADD
    MOV ALU.OUT, R1
    MOV line_h_y_ready, CTRL.JMP
line_h_sub_y:
    MOV R1, ALU.X
    MOV R5, ALU.SUB
    MOV ALU.OUT, R1
line_h_y_ready:
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV putpixel, CTRL.CALL

    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_DX, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV line_done, CTRL.JZ
    MOV GFX_SX, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV 0, R7
    MOV R6, ALU.X
    MOV R7, ALU.CMP
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV line_h_dec_x, CTRL.JZ
    MOV R0, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R0
    MOV line_h_store_x, CTRL.JMP
line_h_dec_x:
    MOV R0, ALU.X
    MOV 1, ALU.SUB
    MOV ALU.OUT, R0
line_h_store_x:
    MOV GFX_TMP0, MEM.ADDR
    MOV R0, MEM.W16
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R0
    MOV GFX_I, MEM.ADDR
    MOV R0, MEM.W16
    MOV line_horizontal_loop, CTRL.JMP

line_vertical_major:
    MOV GFX_DY, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV 0, R7
    MOV R6, ALU.X
    MOV R7, ALU.CMP
    MOV line_single_pixel, CTRL.JZ
line_vertical_loop:
    ; x = x0 +/- (i*dx/dy), y is maintained in TMP1.
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_DX, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV R5, ALU.X
    MOV R6, ALU.MUL
    MOV ALU.OUT, R5
    MOV GFX_DY, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV R5, ALU.X
    MOV R6, ALU.DIV
    MOV ALU.OUT, R5
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_SX, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV 0, R7
    MOV R6, ALU.X
    MOV R7, ALU.CMP
    MOV line_v_sub_x, CTRL.JZ
    MOV R4, ALU.X
    MOV R5, ALU.ADD
    MOV ALU.OUT, R4
    MOV line_v_x_ready, CTRL.JMP
line_v_sub_x:
    MOV R4, ALU.X
    MOV R5, ALU.SUB
    MOV ALU.OUT, R4
line_v_x_ready:
    MOV R4, R0
    MOV GFX_TMP1, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R1
    MOV R4, R0
    MOV putpixel, CTRL.CALL

    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_DY, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV line_done, CTRL.JZ
    MOV GFX_SY, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV 0, R7
    MOV R6, ALU.X
    MOV R7, ALU.CMP
    MOV GFX_TMP1, MEM.ADDR
    MOV MEM.R16, R0
    MOV line_v_dec_y, CTRL.JZ
    MOV R0, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R0
    MOV line_v_store_y, CTRL.JMP
line_v_dec_y:
    MOV R0, ALU.X
    MOV 1, ALU.SUB
    MOV ALU.OUT, R0
line_v_store_y:
    MOV GFX_TMP1, MEM.ADDR
    MOV R0, MEM.W16
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R0
    MOV GFX_I, MEM.ADDR
    MOV R0, MEM.W16
    MOV line_vertical_loop, CTRL.JMP

line_single_pixel:
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R1
    MOV R4, R0
    MOV putpixel, CTRL.CALL
line_done:
    MOV CTRL.RETADDR, CTRL.JMP
.endproc

; rect: GFX_X0=x, GFX_Y0=y, GFX_W=w, GFX_H=h, GFX_COLOR=color.
.proc rect
    MOV GFX_COLOR, MEM.ADDR
    MOV MEM.R16, R0
    MOV gfx_set_color, CTRL.CALL
    MOV GFX_W, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV 0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV rect_done, CTRL.JZ
    MOV GFX_H, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV rect_done, CTRL.JZ

    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_W, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R5, ALU.X
    MOV 1, ALU.SUB
    MOV ALU.OUT, R5
    MOV R5, ALU.X
    MOV R4, ALU.ADD
    MOV ALU.OUT, R5
    MOV R5, R0
    MOV GFX_TMP0, MEM.ADDR
    MOV R0, MEM.W16
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_H, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R5, ALU.X
    MOV 1, ALU.SUB
    MOV ALU.OUT, R5
    MOV R5, ALU.X
    MOV R4, ALU.ADD
    MOV ALU.OUT, R5
    MOV R5, R0
    MOV GFX_TMP1, MEM.ADDR
    MOV R0, MEM.W16

    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R1
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R2
    MOV R4, R0
    MOV hline, CTRL.CALL

    MOV GFX_H, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV 1, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV rect_sides, CTRL.JZ
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R1
    MOV GFX_TMP1, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R2
    MOV R4, R0
    MOV hline, CTRL.CALL
rect_sides:
    MOV GFX_H, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV 2, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV rect_done, CTRL.JNC
    MOV rect_done, CTRL.JZ
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R0
    MOV R0, R5
    MOV GFX_TMP1, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, ALU.X
    MOV 1, ALU.SUB
    MOV ALU.OUT, R0
    MOV R0, R2
    MOV R4, R0
    MOV R5, R1
    MOV vline, CTRL.CALL
    MOV GFX_W, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV 1, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV rect_done, CTRL.JZ
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R0
    MOV R0, R5
    MOV GFX_TMP1, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, ALU.X
    MOV 1, ALU.SUB
    MOV ALU.OUT, R0
    MOV R0, R2
    MOV R4, R0
    MOV R5, R1
    MOV vline, CTRL.CALL
rect_done:
    MOV CTRL.RETADDR, CTRL.JMP
.endproc

; fillrect: same parameter block as rect.
.proc fillrect
    MOV GFX_COLOR, MEM.ADDR
    MOV MEM.R16, R0
    MOV gfx_set_color, CTRL.CALL
    MOV GFX_W, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV 0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV fillrect_done, CTRL.JZ
    MOV GFX_H, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV fillrect_done, CTRL.JZ
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_W, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R5, ALU.X
    MOV 1, ALU.SUB
    MOV ALU.OUT, R5
    MOV R5, ALU.X
    MOV R4, ALU.ADD
    MOV ALU.OUT, R5
    MOV R5, R0
    MOV GFX_TMP0, MEM.ADDR
    MOV R0, MEM.W16
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV GFX_TMP1, MEM.ADDR
    MOV R0, MEM.W16
    MOV GFX_H, MEM.ADDR
    MOV MEM.R16, R0
    MOV GFX_TMP2, MEM.ADDR
    MOV R0, MEM.W16
fillrect_loop:
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R1
    MOV GFX_TMP1, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R2
    MOV R4, R0
    MOV hline, CTRL.CALL
    MOV GFX_TMP2, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, ALU.X
    MOV 1, ALU.SUB
    MOV ALU.OUT, R0
    MOV GFX_TMP2, MEM.ADDR
    MOV R0, MEM.W16
    MOV R0, R4
    MOV 0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV fillrect_done, CTRL.JZ
    MOV GFX_TMP1, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R0
    MOV GFX_TMP1, MEM.ADDR
    MOV R0, MEM.W16
    MOV fillrect_loop, CTRL.JMP
fillrect_done:
    MOV CTRL.RETADDR, CTRL.JMP
.endproc

; Internal helper: plot eight symmetric points. Requires TMP0=x, I=y,
; X0=cx, Y0=cy and current color already selected. Circle must fit on screen.
.proc __gfx_circle_points
    ; (cx+x, cy+y)
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R4, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R4
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R5, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R5
    MOV R4, R0
    MOV R5, R1
    MOV putpixel, CTRL.CALL
    ; (cx+y, cy+x)
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R4, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R4
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R5, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R5
    MOV R4, R0
    MOV R5, R1
    MOV putpixel, CTRL.CALL
    ; (cx-y, cy+x)
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R4, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R4
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R5, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R5
    MOV R4, R0
    MOV R5, R1
    MOV putpixel, CTRL.CALL
    ; (cx-x, cy+y)
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R4, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R4
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R5, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R5
    MOV R4, R0
    MOV R5, R1
    MOV putpixel, CTRL.CALL
    ; (cx-x, cy-y)
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R4, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R4
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R5, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R5
    MOV R4, R0
    MOV R5, R1
    MOV putpixel, CTRL.CALL
    ; (cx-y, cy-x)
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R4, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R4
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R5, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R5
    MOV R4, R0
    MOV R5, R1
    MOV putpixel, CTRL.CALL
    ; (cx+y, cy-x)
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R4, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R4
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R5, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R5
    MOV R4, R0
    MOV R5, R1
    MOV putpixel, CTRL.CALL
    ; (cx+x, cy-y)
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R4, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R4
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R5, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R5
    MOV R4, R0
    MOV R5, R1
    MOV putpixel, CTRL.CALL
    MOV CTRL.RETADDR, CTRL.JMP
.endproc

; circle: GFX_X0=cx, GFX_Y0=cy, GFX_R=r, GFX_COLOR=color.
; Uses an integer scan of the first octant; circle must fit on screen.
.proc circle
    MOV GFX_COLOR, MEM.ADDR
    MOV MEM.R16, R0
    MOV gfx_set_color, CTRL.CALL
    MOV GFX_R, MEM.ADDR
    MOV MEM.R16, R0
    MOV GFX_TMP0, MEM.ADDR
    MOV R0, MEM.W16
    MOV R0, R4
    MOV R4, ALU.X
    MOV R4, ALU.MUL
    MOV ALU.OUT, R4
    MOV R4, R0
    MOV GFX_TMP1, MEM.ADDR
    MOV R0, MEM.W16
    MOV 0, R0
    MOV GFX_I, MEM.ADDR
    MOV R0, MEM.W16
circle_y_loop:
circle_find_x:
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV R4, ALU.X
    MOV R4, ALU.MUL
    MOV ALU.OUT, R4
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R5, ALU.X
    MOV R5, ALU.MUL
    MOV ALU.OUT, R5
    MOV R4, ALU.X
    MOV R5, ALU.ADD
    MOV ALU.OUT, R4
    MOV GFX_TMP1, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV circle_x_ready, CTRL.JNC
    MOV circle_x_ready, CTRL.JZ
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, ALU.X
    MOV 1, ALU.SUB
    MOV ALU.OUT, R0
    MOV GFX_TMP0, MEM.ADDR
    MOV R0, MEM.W16
    MOV circle_find_x, CTRL.JMP
circle_x_ready:
    MOV __gfx_circle_points, CTRL.CALL
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_R, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV circle_done, CTRL.JZ
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R0
    MOV GFX_I, MEM.ADDR
    MOV R0, MEM.W16
    MOV circle_y_loop, CTRL.JMP
circle_done:
    MOV CTRL.RETADDR, CTRL.JMP
.endproc

; fillcircle: same parameters as circle. Circle must fit on screen.
.proc fillcircle
    MOV GFX_COLOR, MEM.ADDR
    MOV MEM.R16, R0
    MOV gfx_set_color, CTRL.CALL
    MOV GFX_R, MEM.ADDR
    MOV MEM.R16, R0
    MOV GFX_TMP0, MEM.ADDR
    MOV R0, MEM.W16
    MOV R0, R4
    MOV R4, ALU.X
    MOV R4, ALU.MUL
    MOV ALU.OUT, R4
    MOV R4, R0
    MOV GFX_TMP1, MEM.ADDR
    MOV R0, MEM.W16
    MOV 0, R0
    MOV GFX_I, MEM.ADDR
    MOV R0, MEM.W16
fillcircle_y_loop:
fillcircle_find_x:
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV R4, ALU.X
    MOV R4, ALU.MUL
    MOV ALU.OUT, R4
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R5, ALU.X
    MOV R5, ALU.MUL
    MOV ALU.OUT, R5
    MOV R4, ALU.X
    MOV R5, ALU.ADD
    MOV ALU.OUT, R4
    MOV GFX_TMP1, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV fillcircle_x_ready, CTRL.JNC
    MOV fillcircle_x_ready, CTRL.JZ
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, ALU.X
    MOV 1, ALU.SUB
    MOV ALU.OUT, R0
    MOV GFX_TMP0, MEM.ADDR
    MOV R0, MEM.W16
    MOV fillcircle_find_x, CTRL.JMP
fillcircle_x_ready:
    ; span at cy+y
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R4, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R4
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R5, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R5
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R6, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R6
    MOV R4, R0
    MOV R5, R1
    MOV R6, R2
    MOV hline, CTRL.CALL
    ; span at cy-y, skip duplicate y=0
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV 0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV fillcircle_cross_spans, CTRL.JZ
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R4, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R4
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R5, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R5
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R6, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R6
    MOV R4, R0
    MOV R5, R1
    MOV R6, R2
    MOV hline, CTRL.CALL
fillcircle_cross_spans:
    ; If x != y, draw the two transposed spans too.
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV fillcircle_next_y, CTRL.JZ
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R4, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R4
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R5, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R5
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R6, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R6
    MOV R4, R0
    MOV R5, R1
    MOV R6, R2
    MOV hline, CTRL.CALL
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV 0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV fillcircle_next_y, CTRL.JZ
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R4, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R4
    MOV GFX_X0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R5, ALU.X
    MOV R0, ALU.ADD
    MOV ALU.OUT, R5
    MOV GFX_Y0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R6
    MOV GFX_TMP0, MEM.ADDR
    MOV MEM.R16, R0
    MOV R6, ALU.X
    MOV R0, ALU.SUB
    MOV ALU.OUT, R6
    MOV R4, R0
    MOV R5, R1
    MOV R6, R2
    MOV hline, CTRL.CALL
fillcircle_next_y:
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R4
    MOV GFX_R, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, R5
    MOV R4, ALU.X
    MOV R5, ALU.CMP
    MOV fillcircle_done, CTRL.JZ
    MOV GFX_I, MEM.ADDR
    MOV MEM.R16, R0
    MOV R0, ALU.X
    MOV 1, ALU.ADD
    MOV ALU.OUT, R0
    MOV GFX_I, MEM.ADDR
    MOV R0, MEM.W16
    MOV fillcircle_y_loop, CTRL.JMP
fillcircle_done:
    MOV CTRL.RETADDR, CTRL.JMP
.endproc
