; A+X+Y cost-optimized memory copy kernels.
; X = source, Y = destination. Count/control can live in memory or caller logic.
forward_byte:
    LDA8 [X+]
    STA8 [Y+]
    RET

; X and Y point one byte past the regions.
backward_byte:
    LDA8 [-X]
    STA8 [-Y]
    RET
