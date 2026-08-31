# Memory-register Assembly Programming Manual


> Current video model: system memory and video memory are separate 16-bit spaces. Ordinary memory instructions never cross into video memory. See `../../../docs/PLATFORM.md` for the authoritative map and the architecture-specific video instruction forms.

The Memory-register CPU is a PIC-inspired, cost-oriented 16-bit architecture without historical banked-memory restrictions. Arithmetic centers on W and direct zero-page file operands. Two FSR registers provide full 64 KiB indirect access.

Programs normally reserve `0x0000..0x00EF` for fast variables and compiler data, `0x00F0..0x00FF` for compiler/scratch use, and load code at `0x0100` or above.

## Procedure blocks and unused-code removal

Public/callable routines should be written as `.proc NAME` ... `.endproc` blocks. `.entry NAME` makes the program entry procedure a reachability root; `.keep NAME` keeps hardware callbacks or standalone library-fragment procedures explicitly. After `.include` and `.equ` expansion, the assembler removes `.proc` blocks that are not reachable from these roots or from symbolic references in live code. Ordinary labels inside a procedure remain local control-flow labels and do not define separate collectible procedures.


## Destination flag model

Direct ALU operations can write either W or the file operand:

```asm
MOV16 0x10,W
ADD   0x12,W     ; W = W + file[0x12]
ADD   0x14,F     ; file[0x14] = file[0x14] + W
```

The assembler automatically emits a one-byte hot encoding when the operation and file address permit it.

## Indirect addressing

```asm
FSR0I source
FSR1I destination
LDB0+             ; W = *FSR0++
STB1+             ; *FSR1++ = W
```

Backward overlap-safe copying starts both pointers one byte beyond their last copied element and uses pre-decrement:

```asm
LDB0-
STB1-
```

No block-copy instruction is needed; the primitive remains useful for strings, framebuffers and buffers.

## Zero page and full memory

Direct file instructions access only `0x0000..0x00FF`. For addresses elsewhere, load an FSR and use indirect access. This avoids a bank/page-base state register and keeps call conventions simple.

## Timer / interrupt quick reference

The shared machine provides a 32-bit virtual clock, one 16-bit timer, and timer/VSYNC/keyboard IRQ sources at `0xFF12..0xFF1F`. Configure the vector and source mask while interrupts are disabled, acknowledge handled bits through `IRQ_ACK` (`0xFF14`), then return with `IRET`. See the project-level `../../../docs/PLATFORM.md` for the normative MMIO semantics.

## Graphics library

`graphics.asm` exports the fast `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, and `vline` primitives plus the higher-level `line`, `rect`, `fillrect`, `circle`, and `fillcircle` procedures. ABI: W=color; palette: FSR0 -> 4-byte table; putpixel: FSR0=x,FSR1=y; clear: W=color; hline: FSR0=x0,FSR1=x1,W=y; vline: FSR0=x,FSR1=y0,W=y1. Unused procedures are removed by procedure-GC.

Shapes with five or more logical parameters use the same 16-bit graphics parameter block on every ISA: `GFX_X0=0x00C0`, `GFX_Y0=0x00C2`, `GFX_X1=0x00C4`, `GFX_Y1=0x00C6`, `GFX_W=0x00C8`, `GFX_H=0x00CA`, `GFX_R=0x00CC`, `GFX_COLOR=0x00CE`. `0x00B0..0x00BE` is internal virtual-register scratch on some targets; `0x00D0..0x00FA` is additional graphics scratch/current-colour storage. Therefore `graphics.asm` reserves the full `0x00B0..0x00FA` range. `line` reads `(x0,y0,x1,y1,color)`, `rect/fillrect` read `(x,y,w,h,color)`, and `circle/fillcircle` read `(cx,cy,r,color)`. Procedure-GC removes unused shape procedures and dependencies.

