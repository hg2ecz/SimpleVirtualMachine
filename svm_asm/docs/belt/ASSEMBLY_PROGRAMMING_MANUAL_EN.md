# Belt16 assembly programming manual

Belt16 names the eight most recent 16-bit results `b0..b7`; `b0` is always the newest. Every result-producing instruction ages older values by one position. There is no general-purpose register file.

```asm
.load 0x0100
.entry start
.proc start
    LDI 10
    LDI 20
    ADD b1,b0
    ST16A 0x6000,b0
    HALT
.endproc
```

Absolute memory uses `LD8A/LD16A` and `ST8A/ST16A`. Pointer memory uses `LD8/LD16 [bN]` and `ST8/ST16 [bA],bV`. Video memory uses `VLD8/VLD16` and `VST8/VST16`.

`PUSH bN` and `POP` are primarily compiler/assembly convenience primitives. `POP` produces a result and therefore places it on the belt.

`CMP bA,bB` also produces a result (`a-b`) and updates `Z/N/C`; the result can be followed by `JZ/JNZ/JC/JNC/JN/JNN`.

## Procedure blocks and unused-code removal

Public/callable routines should be written as `.proc NAME` ... `.endproc` blocks. `.entry NAME` makes the program entry procedure a reachability root; `.keep NAME` keeps hardware callbacks or standalone library-fragment procedures explicitly. After `.include` and `.equ` expansion, the assembler removes `.proc` blocks that are not reachable from these roots or from symbolic references in live code. Ordinary labels inside a procedure remain local control-flow labels and do not define separate collectible procedures.

## Graphics library

`graphics.asm` exports the fast `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, and `vline` primitives plus the higher-level `line`, `rect`, `fillrect`, `circle`, and `fillcircle` procedures. ABI: b0=color; palette: b0..b3; putpixel: b0=x, b1=y; clear: b0=color; hline: b0=x0,b1=x1,b2=y; vline: b0=x,b1=y0,b2=y1. Unused procedures are removed by procedure-GC.

Shapes with five or more logical parameters use the same 16-bit graphics parameter block on every ISA: `GFX_X0=0x00C0`, `GFX_Y0=0x00C2`, `GFX_X1=0x00C4`, `GFX_Y1=0x00C6`, `GFX_W=0x00C8`, `GFX_H=0x00CA`, `GFX_R=0x00CC`, `GFX_COLOR=0x00CE`. `0x00B0..0x00BE` is internal virtual-register scratch on some targets; `0x00D0..0x00FA` is additional graphics scratch/current-colour storage. Therefore `graphics.asm` reserves the full `0x00B0..0x00FA` range. `line` reads `(x0,y0,x1,y1,color)`, `rect/fillrect` read `(x,y,w,h,color)`, and `circle/fillcircle` read `(cx,cy,r,color)`. Procedure-GC removes unused shape procedures and dependencies.

