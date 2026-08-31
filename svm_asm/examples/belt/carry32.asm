; Belt16 32-bit addition using the carry flag.
; 0x0001FFFF + 0x00020001 = 0x00040000
; Output: low word at 0x6000, high word at 0x6002.
.load 0x0100
.entry start

.proc start
    LDI 0xFFFF
    LDI 0x0001
    ADD b1,b0       ; b0=0x0000, C=1
    PUSH b0         ; preserve low result; PUSH does not change C

    LDI 0x0001
    LDI 0x0002
    ADC b1,b0       ; b0=0x0004
    ST16A 0x6002,b0

    POP             ; b0=low result
    ST16A 0x6000,b0
    HALT
.endproc
