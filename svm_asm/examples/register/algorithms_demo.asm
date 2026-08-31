; Hand-written ASM demonstration only. Portable algorithms live in svm_c/lib.
.include "algorithms_demo.asm"
.include "format.asm"
.load 0x0200
.entry start

.proc start
    MOVI R0, 0xF0F1
    CALL popcount16_demo
    CALL putu16
    HALT
.endproc
