; Compile with: svm-asm -I svm_asm/lib/register register this.asm out.svm
.include "console.asm"

.load 0x0200
.entry start
start:
    MOVI R0, 65
    CALL putc
    CALL newline
    HALT
