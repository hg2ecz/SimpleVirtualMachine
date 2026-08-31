; Register-ISA demonstration helper library.
; The portable algorithm library is intentionally maintained in svm_c/lib.
; This file only demonstrates how a hand-written ASM helper can coexist with it.

; popcount16_demo: R0=value -> R0=number of set bits. Clobbers R1,R2,R3.
.proc popcount16_demo
    MOVI R3, 0
    MOVI R2, 1
popcount16_demo_loop:
    CMPI R0, 0
    JZ popcount16_demo_done
    MOV R1, R0
    AND R1, R2
    ADD R3, R1
    SHR1 R0
    JMP popcount16_demo_loop
popcount16_demo_done:
    MOV R0, R3
    RET
.endproc
