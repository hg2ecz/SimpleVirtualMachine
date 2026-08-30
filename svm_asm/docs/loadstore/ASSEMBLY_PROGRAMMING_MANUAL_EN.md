# Load/Store assembly programming manual

The machine is the strict RISC/load-store control point: computation happens in registers and memory is accessed only by explicit load/store instructions.

```asm
MOVI R1, 0x1000
LOAD16 R0, [R1+0]
ADDI R0, 1
STORE16 [R1+0], R0
```

Multiword arithmetic chains the `C` flag:

```asm
ADD  R0,R0,R2
ADC  R1,R1,R3
```

A 32-bit right shift can use:

```asm
SHR1 R1
RCR1 R0
```

`SUBI` is a native long-immediate subtraction; do not replace it with `ADDI -imm` when following code observes the `C` no-borrow state.

VRAM is a separate address space and uses `VLOAD*`/`VSTORE*`. Platform MMIO addresses are documented in `../../../docs/PLATFORM.md`.
