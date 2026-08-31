; Belt16 video/MMIO example.
; Writes one packed framebuffer byte and asks the text generator to draw 'A'.
.load 0x0100
.entry start

.proc start
    ; Four 2-bit pixels, all palette slot 3.
    LDI 0x0000      ; b0=VRAM address
    LDI 0x00FF      ; b0=packed pixels, b1=VRAM address
    VST8 [b1],b0

    ; Video text-generator position and character.
    LDI 4
    ST8A 0xFF02,b0  ; TEXT_X
    LDI 3
    ST8A 0xFF03,b0  ; TEXT_Y
    LDI 65
    ST8A 0xFF06,b0  ; TEXT_CHAR = 'A'

    HALT
.endproc
