.load 0x0100
.include "graphics.asm"
.entry start
.proc start
    0 CALL clear
    10 GFX_X0 STORE16
    10 GFX_Y0 STORE16
    309 GFX_X1 STORE16
    189 GFX_Y1 STORE16
    1 GFX_COLOR STORE16
    CALL line
    120 GFX_X0 STORE16
    60 GFX_Y0 STORE16
    80 GFX_W STORE16
    50 GFX_H STORE16
    2 GFX_COLOR STORE16
    CALL fillrect
    HALT
.endproc
