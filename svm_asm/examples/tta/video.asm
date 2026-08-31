; Write one packed framebuffer byte through VRAM transport ports.
.entry start
.proc start
    MOV 0, VMEM.ADDR
    MOV 0xFF, VMEM.W8
    HALT
.endproc
