; The register standard-library directory is searched automatically.
.include "graphics.asm"
.load 0x0100
.entry start
.proc start
    ; slot0=black, slot1=bright blue, slot2=yellow, slot3=white
    MOVI R0, 0
    MOVI R1, 9
    MOVI R2, 14
    MOVI R3, 15
    CALL gfx_set_palette
    MOVI R0, 0
    CALL clear

    ; line(10,309,10,189,1) through the shared high-level parameter block
    MOVI R0, 10
    ZSTORE16 GFX_X0
    MOVI R0, 10
    ZSTORE16 GFX_Y0
    MOVI R0, 309
    ZSTORE16 GFX_X1
    MOVI R0, 189
    ZSTORE16 GFX_Y1
    MOVI R0, 1
    ZSTORE16 GFX_COLOR
    CALL line

    ; fillrect(120,60,80,50,2)
    MOVI R0, 120
    ZSTORE16 GFX_X0
    MOVI R0, 60
    ZSTORE16 GFX_Y0
    MOVI R0, 80
    ZSTORE16 GFX_W
    MOVI R0, 50
    ZSTORE16 GFX_H
    MOVI R0, 2
    ZSTORE16 GFX_COLOR
    CALL fillrect
    HALT
.endproc
