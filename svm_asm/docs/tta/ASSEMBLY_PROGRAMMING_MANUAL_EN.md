# TTA16 assembly programming manual

On TTA16, the basic programming concept is not `ADD` or `LOAD`, but a `MOV source,destination-port` transport.

## Procedure blocks and unused-code removal

Public/callable routines should be written as `.proc NAME` ... `.endproc` blocks. `.entry NAME` makes the program entry procedure a reachability root; `.keep NAME` keeps hardware callbacks or standalone library-fragment procedures explicitly. After `.include` and `.equ` expansion, the assembler removes `.proc` blocks that are not reachable from these roots or from symbolic references in live code. Ordinary labels inside a procedure remain local control-flow labels and do not define separate collectible procedures.


## Arithmetic

```asm
MOV R0, ALU.X
MOV R1, ALU.ADD
MOV ALU.OUT, R2
```

`ALU.X` stores the first operand. Writing `ALU.ADD` triggers addition. The result appears at the `ALU.OUT` source port.

## Memory

```asm
MOV 0x6000, MEM.ADDR
MOV 123, MEM.W16
MOV MEM.R16, R0
```

## Control

```asm
JNZ loop
CALL function
RET
```

These are assembler convenience forms for transports to `CTRL.*` ports.

See also `INSTRUCTION_REFERENCE_EN.md` and `../../../docs/TTA_ISA_SPEC_HU.md`.

## Graphics library

`graphics.asm` exports the fast `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, and `vline` primitives plus the higher-level `line`, `rect`, `fillrect`, `circle`, and `fillcircle` procedures. ABI: R0=color; palette: R0..R3; putpixel: R0=x,R1=y; clear: R0=color; hline: R0=x0,R1=x1,R2=y; vline: R0=x,R1=y0,R2=y1. Unused procedures are removed by procedure-GC.

Shapes with five or more logical parameters use the same 16-bit graphics parameter block on every ISA: `GFX_X0=0x00C0`, `GFX_Y0=0x00C2`, `GFX_X1=0x00C4`, `GFX_Y1=0x00C6`, `GFX_W=0x00C8`, `GFX_H=0x00CA`, `GFX_R=0x00CC`, `GFX_COLOR=0x00CE`. `0x00B0..0x00BE` is internal virtual-register scratch on some targets; `0x00D0..0x00FA` is additional graphics scratch/current-colour storage. Therefore `graphics.asm` reserves the full `0x00B0..0x00FA` range. `line` reads `(x0,y0,x1,y1,color)`, `rect/fillrect` read `(x,y,w,h,color)`, and `circle/fillcircle` read `(cx,cy,r,color)`. Procedure-GC removes unused shape procedures and dependencies.

