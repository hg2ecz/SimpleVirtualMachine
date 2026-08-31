.load 0x0100
.include "graphics.asm"
.entry start
.proc start
    LEA A0, 0
    CALL clear
    MOV16 [GFX_X0], 10
    MOV16 [GFX_Y0], 10
    MOV16 [GFX_X1], 309
    MOV16 [GFX_Y1], 189
    MOV16 [GFX_COLOR], 1
    CALL line
    MOV16 [GFX_X0], 120
    MOV16 [GFX_Y0], 60
    MOV16 [GFX_W], 80
    MOV16 [GFX_H], 50
    MOV16 [GFX_COLOR], 2
    CALL fillrect
    HALT
.endproc
