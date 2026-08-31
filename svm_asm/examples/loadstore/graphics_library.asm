; The register standard-library directory is searched automatically.
.include "graphics.asm"
.load 0x0100
.entry start
.proc start
    ; slot0=black, slot1=bright blue, slot2=yellow, slot3=white
    LDI R0, 0
    LDI R1, 9
    LDI R2, 14
    LDI R3, 15
    CALL gfx_set_palette
    LDI R0, 0
    CALL clear

    ; line(10,309,10,189,1) through the shared high-level parameter block
    LDI R0, 10
    ZSTORE16 GFX_X0
    LDI R0, 10
    ZSTORE16 GFX_Y0
    LDI R0, 309
    ZSTORE16 GFX_X1
    LDI R0, 189
    ZSTORE16 GFX_Y1
    LDI R0, 1
    ZSTORE16 GFX_COLOR
    CALL line

    ; fillrect(120,60,80,50,2)
    LDI R0, 120
    ZSTORE16 GFX_X0
    LDI R0, 60
    ZSTORE16 GFX_Y0
    LDI R0, 80
    ZSTORE16 GFX_W
    LDI R0, 50
    ZSTORE16 GFX_H
    LDI R0, 2
    ZSTORE16 GFX_COLOR
    CALL fillrect
    HALT
.endproc
