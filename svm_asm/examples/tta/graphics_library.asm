.load 0x0100
.include "graphics.asm"
.entry start
.proc start
    MOV 0, R0
    CALL clear
    MOV 10, R0
    MOV GFX_X0, MEM.ADDR
    MOV R0, MEM.W16
    MOV 10, R0
    MOV GFX_Y0, MEM.ADDR
    MOV R0, MEM.W16
    MOV 309, R0
    MOV GFX_X1, MEM.ADDR
    MOV R0, MEM.W16
    MOV 189, R0
    MOV GFX_Y1, MEM.ADDR
    MOV R0, MEM.W16
    MOV 1, R0
    MOV GFX_COLOR, MEM.ADDR
    MOV R0, MEM.W16
    CALL line
    MOV 120, R0
    MOV GFX_X0, MEM.ADDR
    MOV R0, MEM.W16
    MOV 60, R0
    MOV GFX_Y0, MEM.ADDR
    MOV R0, MEM.W16
    MOV 80, R0
    MOV GFX_W, MEM.ADDR
    MOV R0, MEM.W16
    MOV 50, R0
    MOV GFX_H, MEM.ADDR
    MOV R0, MEM.W16
    MOV 2, R0
    MOV GFX_COLOR, MEM.ADDR
    MOV R0, MEM.W16
    CALL fillrect
    HALT
.endproc
