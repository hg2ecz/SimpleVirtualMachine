; Overlap-safe byte memmove building blocks.
; .keep makes this library-style example emit both kernels when assembled directly.
.keep forward
.keep backward

; Forward: R0=src, R1=dst, R2=count.
.proc forward
    CMP R2, R7       ; R7 expected 0
    JZ forward_done
loop_f:
    LOAD8 R3, [R0+]
    STORE8 [R1+], R3
    DEC R2
    JNZ loop_f
forward_done:
    RET
.endproc

; Backward: R0=src+count, R1=dst+count, R2=count.
.proc backward
    CMP R2, R7
    JZ backward_done
loop_b:
    LOAD8 R3, [-R0]
    STORE8 [-R1], R3
    DEC R2
    JNZ loop_b
backward_done:
    RET
.endproc
