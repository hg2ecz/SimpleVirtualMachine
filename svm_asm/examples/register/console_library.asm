; The register standard-library directory is searched automatically.
.include "console.asm"

.load 0x0200
.entry start
.proc start
    MOVI R0, 65
    CALL putc
    CALL newline
    HALT
.endproc
