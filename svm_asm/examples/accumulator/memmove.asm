; A+X+Y cost-optimized memory copy kernels.
; Keep both because this is a standalone library-fragment example.
.keep forward_byte
.keep backward_byte
; X = source, Y = destination. Count/control can live in memory or caller logic.
.proc forward_byte
    LDA8 [X+]
    STA8 [Y+]
    RET
.endproc

; X and Y point one byte past the regions.
.proc backward_byte
    LDA8 [-X]
    STA8 [-Y]
    RET
.endproc
