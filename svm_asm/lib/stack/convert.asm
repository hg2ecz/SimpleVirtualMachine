\ Stack conversion demo. Uses the existing decimal formatter and adds raw float aliases.
.include "format.asm"
.proc f16_to_decstr
 CALL putu16
 RET
.endproc
