; Draw "HELLO" through the common TEXT_CHAR MMIO character generator.
; Pixel slots 1 and 3 are mapped to blue and yellow from the fixed 16-colour palette.
.load 0x0200
.entry start

start:
    MOVI R1, 0xFF0D      ; palette slot 1 selector
    MOVI R2, 1           ; blue
    STORE8 [R1], R2
    MOVI R1, 0xFF0F      ; palette slot 3 selector
    MOVI R2, 14          ; yellow
    STORE8 [R1], R2

    MOVI R1, 0xFF03      ; TEXT_Y
    MOVI R2, 4
    STORE8 [R1], R2
    MOVI R1, 0xFF04      ; TEXT_FG = framebuffer slot 3
    MOVI R2, 3
    STORE8 [R1], R2
    MOVI R1, 0xFF05      ; TEXT_BG = framebuffer slot 1
    MOVI R2, 1
    STORE8 [R1], R2

    MOVI R1, 0xFF02      ; TEXT_X
    MOVI R2, 7
    STORE8 [R1], R2
    MOVI R3, 0xFF06      ; TEXT_CHAR
    MOVI R0, 72          ; H
    STORE8 [R3], R0

    INC R2
    STORE8 [R1], R2
    MOVI R0, 69          ; E
    STORE8 [R3], R0

    INC R2
    STORE8 [R1], R2
    MOVI R0, 76          ; L
    STORE8 [R3], R0

    INC R2
    STORE8 [R1], R2
    STORE8 [R3], R0

    INC R2
    STORE8 [R1], R2
    MOVI R0, 79          ; O
    STORE8 [R3], R0
    HALT
