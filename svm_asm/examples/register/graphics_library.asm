; Assemble with -I svm_asm/lib/register
.include "graphics.asm"
.load 0x0200
.entry start
start:
    ; slot0=black, slot1=bright blue, slot2=yellow, slot3=white
    MOVI R0, 0
    MOVI R1, 9
    MOVI R2, 14
    MOVI R3, 15
    CALL gfx_set_palette
    MOVI R0, 0
    CALL clear
    MOVI R0, 2
    CALL gfx_set_color
    MOVI R0, 20
    MOVI R1, 300
    MOVI R2, 100
    CALL hline
    MOVI R0, 160
    MOVI R1, 20
    MOVI R2, 180
    CALL vline
    HALT
