; Fill the single framebuffer in separate video space.
; 320x200, packed 2bpp: 16000 bytes at video 0x0000..0x3E7F.
.load 0x0000
.entry start
.proc start
    MOVI R0, 0x0000
    MOVI R1, 16000
    MOVI R2, 0
    MOVI R3, 1
loop:
    VSTORE8P [R0+], R2
    ADD R2, R3
    DEC R1
    JNZ loop
    HALT
.endproc
