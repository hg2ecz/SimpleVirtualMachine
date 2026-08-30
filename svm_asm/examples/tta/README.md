# TTA16 assembly examples

The TTA16 core instruction is `MOV source,destination`. Writing a value to an
ALU trigger port starts the operation; the result is read from `ALU.OUT`.
Memory and control operations use the same transport principle.

Examples:
- `basic.asm` - arithmetic transport sequence
- `memory.asm` - RAM address/read/write ports
- `loop.asm` - ALU flags plus conditional control transport
- `call.asm` - control stack with CALL/RET assembler conveniences
- `carry32.asm` - 32-bit carry chain with ADD/ADC
- `video.asm` - VRAM port access

Assemble with:

    svm-asm tta svm_asm/examples/tta/basic.asm basic.svt
