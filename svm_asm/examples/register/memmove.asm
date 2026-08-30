; Overlap-safe byte memmove building blocks.
; Forward: R0=src, R1=dst, R2=count.
forward:
    CMP R2, R7       ; R7 expected 0
    JZ done
loop_f:
    LOAD8 R3, [R0+]
    STORE8 [R1+], R3
    DEC R2
    JNZ loop_f
    RET

; Backward: R0=src+count, R1=dst+count, R2=count.
backward:
    CMP R2, R7
    JZ done
loop_b:
    LOAD8 R3, [-R0]
    STORE8 [-R1], R3
    DEC R2
    JNZ loop_b
done:
    RET
