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

## Procedure blocks and unused-code removal

Public/callable routines should be written as `.proc NAME` ... `.endproc` blocks. `.entry NAME` makes the program entry procedure a reachability root; `.keep NAME` keeps hardware callbacks or standalone library-fragment procedures explicitly. After `.include` and `.equ` expansion, the assembler removes `.proc` blocks that are not reachable from these roots or from symbolic references in live code. Ordinary labels inside a procedure remain local control-flow labels and do not define separate collectible procedures.

## Graphics library

`graphics.asm` exports the fast `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, and `vline` primitives plus the higher-level `line`, `rect`, `fillrect`, `circle`, and `fillcircle` procedures. ABI: R0=color; palette: R0..R3; putpixel: R0=x,R1=y; clear: R0=color; hline: R0=x0,R1=x1,R2=y; vline: R0=x,R1=y0,R2=y1. Unused procedures are removed by procedure-GC.

Shapes with five or more logical parameters use the same 16-bit graphics parameter block on every ISA: `GFX_X0=0x00C0`, `GFX_Y0=0x00C2`, `GFX_X1=0x00C4`, `GFX_Y1=0x00C6`, `GFX_W=0x00C8`, `GFX_H=0x00CA`, `GFX_R=0x00CC`, `GFX_COLOR=0x00CE`. `0x00B0..0x00BE` is internal virtual-register scratch on some targets; `0x00D0..0x00FA` is additional graphics scratch/current-colour storage. Therefore `graphics.asm` reserves the full `0x00B0..0x00FA` range. `line` reads `(x0,y0,x1,y1,color)`, `rect/fillrect` read `(x,y,w,h,color)`, and `circle/fillcircle` read `(cx,cy,r,color)`. Procedure-GC removes unused shape procedures and dependencies.

