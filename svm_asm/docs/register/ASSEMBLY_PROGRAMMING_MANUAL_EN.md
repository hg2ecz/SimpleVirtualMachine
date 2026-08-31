# SVM Register CPU – Assembly Programming Manual


> Current video model: system memory and video memory are separate 16-bit spaces. Ordinary memory instructions never cross into video memory. See `../../../docs/PLATFORM.md` for the authoritative map and the architecture-specific video instruction forms.

This document is the programming manual for the current cost-optimized **SVM Register CPU ISA v2 / executable v5**. Its purpose is not merely to list opcodes, but to explain how to program the machine efficiently.

## Procedure blocks and unused-code removal

Public/callable routines should be written as `.proc NAME` ... `.endproc` blocks. `.entry NAME` makes the program entry procedure a reachability root; `.keep NAME` keeps hardware callbacks or standalone library-fragment procedures explicitly. After `.include` and `.equ` expansion, the assembler removes `.proc` blocks that are not reachable from these roots or from symbolic references in live code. Ordinary labels inside a procedure remain local control-flow labels and do not define separate collectible procedures.


## 1. Programmer's model

The CPU is a 16-bit, little-endian machine with a 64 KiB address space.

- 8 general-purpose 16-bit registers: `R0..R7`
- 16-bit program counter `PC`
- internal stack pointer `SP`
- three status flags: `Z`, `N`, `C`
- all addresses are 16-bit
- 16-bit memory values are little-endian

`R0..R7` are functionally equivalent. However, `R0..R3` form the **compact register subset**: several common two-register instructions have a 1-byte encoding when both operands are in this subset. Therefore, hot inner-loop variables should preferably be kept in `R0..R3`.

## 2. Assembly source structure

Minimal program:

```asm
.load 0x0200
.entry start
.proc start
    MOVI R0, 1
    HALT
.endproc
```

### Directives

- `.load address` – program load address
- `.entry procedure` – entry procedure and procedure-GC root
- `.proc name` / `.endproc` – collectible procedure block
- `.keep name` – explicitly retain a procedure
- `.include "file"` – include source library
- `.equ name, value` – symbolic constant

### Comments

A semicolon `;` starts a comment extending to the end of the line.

### Numbers

Accepted examples:

```text
1234
0x1234
0xFF06
```

Registers are written as `R0..R7`. Mnemonics are case-insensitive.

## 3. Recommended register use

The hardware does not impose a calling convention, but the following convention is practical for hand-written assembly:

| Register | Recommended role |
|---|---|
| R0 | primary pointer / argument |
| R1 | secondary pointer / argument |
| R2 | counter / temporary data |
| R3 | temporary data / result |
| R4..R6 | longer-lived variables |
| R7 | scratch register or local temporary |

This is only a recommendation. Favoring `R0..R3` generally reduces code size.

## 4. Compact encoding

The following instructions are 1 byte when both operands are in `R0..R3`:

```text
MOV ADD SUB CMP LOAD8 STORE8 XOR
```

For example:

```asm
ADD R0, R1
```

is 1 byte, while:

```asm
ADD R4, R5
```

is 2 bytes.

The assembler automatically chooses the shorter form; there is no separate compact mnemonic.

All single-register operations are 1 byte for all eight registers:

```text
NOT NEG INC DEC SHL1 SHR1 PUSH POP
```

Immediate-register instructions are 3 bytes:

```text
MOVI ADDI SUBI CMPI
```

## 5. Data movement

### MOV

```asm
MOV Rd, Rs
```

Performs `Rd = Rs`.

### MOVI

```asm
MOVI Rd, 0x1234
```

Loads a 16-bit immediate constant.

### CLR pseudo-instruction

```asm
CLR R2
```

The assembler emits the existing `XOR R2,R2` form. No dedicated opcode is consumed; XOR remains in the general two-register family.

## 6. Arithmetic

```text
ADD Rd,Rs
ADDI Rd,imm16
SUB Rd,Rs
SUBI Rd,imm16
MUL Rd,Rs
DIV Rd,Rs
MOD Rd,Rs
NEG Rd
INC Rd
DEC Rd
```

Arithmetic is 16-bit and wraps modulo 65536.

`DIV` and `MOD` are unsigned. Division by zero is a runtime error.

The assembler automatically shortens:

```asm
ADDI R0, 1
```

to `INC R0`, and:

```asm
SUBI R0, 1
```

to `DEC R0`.

## 7. Logical and shift operations

```text
AND Rd,Rs
OR  Rd,Rs
XOR Rd,Rs
NOT Rd
SHL Rd,Rs
SHR Rd,Rs
SHL1 Rd
SHR1 Rd
```

For variable shifts, the shift count is `Rs & 15`.

`SHL1` and `SHR1` are especially useful for pixel, mask, and address calculations because they are 1-byte instructions.

## 8. Flags

The CPU maintains three flags:

- `Z` – result is zero
- `N` – bit 15 of the result is set
- `C` – carry for addition, no-borrow for subtraction/comparison

There is no signed-overflow (`V`) flag.

### Comparison

```asm
CMP  R0, R1
CMPI R0, 100
```

The registers are not modified. Flags are updated as if `R0-R1` or `R0-100` had been evaluated.

### TEST pseudo-instruction

```asm
TEST R0
```

The assembler emits `OR R0,R0`. This is useful for `Z/N` testing without a dedicated opcode.

## 9. Conditional and unconditional branches

```text
JMP  label
CALL label
JZ   label
JNZ  label
JC   label
JNC  label
JN   label
JNN  label
RET
```

Branches carry a 16-bit absolute destination address.

Typical loop:

```asm
    MOVI R0, 100
loop:
    ; ...
    DEC R0
    JNZ loop
```

## 10. CALL, RET, and the hardware stack

`CALL` pushes the return address onto the CPU's internal memory stack and jumps to the target. `RET` restores the return address from the same stack.

`PUSH Rn` and `POP Rn` use this same hardware stack, so they must be balanced carefully inside subroutines.

Example:

```asm
CALL add_one
HALT

add_one:
    INC R0
    RET
```

Saving registers:

```asm
worker:
    PUSH R4
    PUSH R5
    ; ...
    POP R5
    POP R4
    RET
```

## 11. Normal indirect memory access

```asm
LOAD8  Rd, [Ra]
LOAD16 Rd, [Ra]
STORE8 [Ra], Rs
STORE16 [Ra], Rs
```

- `LOAD8` zero-extends a byte to 16 bits.
- `STORE8` writes only the low 8 bits of the source.
- `LOAD16/STORE16` operate on little-endian 16-bit cells.
- 16-bit access at address `0xFFFF` is invalid.

## 12. Post-increment memory access

Cost-optimized primitives for linear memory traversal:

```asm
LOAD8  R2, [R0+]     ; R2 = mem8[R0],  R0 += 1
STORE8 [R1+], R2     ; mem8[R1] = R2,  R1 += 1
LOAD16 R2, [R0+]     ; R2 = mem16[R0], R0 += 2
STORE16 [R1+], R2    ; mem16[R1] = R2, R1 += 2
```

For post-increment loads, the destination register and address register must be different.

### Copying 256 bytes

```asm
.load 0x0200
.entry start
.proc start
    MOVI R0, 0x3000
    MOVI R1, 0x4000
    MOVI R2, 256
copy:
    LOAD8  R3, [R0+]
    STORE8 [R1+], R3
    DEC R2
    JNZ copy
    HALT
.endproc
```

This is the recommended form for linear buffers, strings, and framebuffer operations.

## 13. Memory map

Important regions in the current microcomputer profile:

| Address | Function |
|---|---|
| `0x0000..0xFAFF` | program/data RAM (below the upper 1 KiB runtime-stack convention) |
| `0xFF00` | `KEY_STATUS` |
| `0xFF01` | `KEY_CODE` |
| `0xFF02..0xFF06` | text X/Y, FG/BG and `TEXT_CHAR` |
| `0xFF0B` | VSYNC counter |
| `0xFF0C..0xFF0F` | four 4-bit colour selectors into the fixed 16-colour master palette |
| `0xFB00..0xFEFF` | CPU stack |
| separate video space `0x0000..0x3E7F` | 16,000-byte framebuffer |
| separate video space `0x3E80..0x3FFF` | 384 reserved VRAM bytes |


## 14. Video: 320x200, 2 bpp, one 16 KiB VRAM

The framebuffer is 320x200 pixels, 2 bits per pixel and four pixels per byte. It occupies 16,000 bytes of a separate 16 KiB data-only VRAM. There is no framebuffer bank or swap mechanism.

Each 2-bit pixel selects slot 0..3. MMIO `0xFF0C..0xFF0F` selects one of 16 fixed master colours for each slot. Only the low nibble is used.

The dedicated `VLOAD/VSTORE` instructions access this video data space; instruction fetch never does.

## 15. Character generator

The 40x25 text grid uses the video device's internal read-only 8x8 glyph ROM. Writing `TEXT_CHAR` at `0xFF06` expands the selected glyph into the framebuffer using `TEXT_FG`/`TEXT_BG` slot numbers. The glyph ROM is not mapped into the CPU address space.

There is no firmware call for text output. Set `TEXT_X/TEXT_Y` and write the character byte directly to `TEXT_CHAR`.

## 16. Keyboard

`KEY_STATUS` (`0xFF00`) indicates whether a character is available; `KEY_CODE` (`0xFF01`) contains the character code.

Simple polling loop:

```asm
    MOVI R0, 0xFF00
    MOVI R1, 0xFF01
wait_key:
    LOAD8 R2, [R0]
    TEST R2
    JZ wait_key
    LOAD8 R3, [R1]
```

## 17. Code-size optimization rules

1. Keep the most frequently used inner-loop operands in `R0..R3`.
2. Prefer `SHL1/SHR1` for one-bit shifts instead of loading a shift-count register.
3. Use `[Rn+]` for linear pointer traversal instead of a separate `INC`.
4. `ADDI Rn,1` and `SUBI Rn,1` may be written naturally; the assembler shortens them.
5. `CLR` and `TEST` add no opcode cost.
6. Keep frequently reused MMIO addresses in registers when that saves repeated immediate loads.
7. Free opcode space is not a design goal; prefer existing short primitives over adding features.

## 18. Common mistakes

- `LOAD8 R0,[R0+]` – invalid because the loaded value and updated pointer would need the same register.
- 16-bit load/store at `0xFFFF` – invalid.
- the internal character ROM is not CPU-addressable; use the `TEXT_CHAR` MMIO register for character drawing.
- forgetting `RET` after `CALL`, or unbalanced `PUSH/POP` – can corrupt return flow.
- interpreting `C` as signed overflow – this CPU has no `V` flag.
- treating `TEXT_FG/TEXT_BG` as master-palette indices instead of 2-bit framebuffer slots.

## 19. Complete example: draw text with the internal character ROM

Use `TEXT_X/TEXT_Y`, select framebuffer colour slots with `TEXT_FG/TEXT_BG`, then write the character byte to `TEXT_CHAR` (`0xFF06`). The glyph ROM is internal to the video device.

## 20. Related documents

- `../README.md` – assembler/ISA entry point
- `INSTRUCTION_REFERENCE_EN.md` – authoritative hexadecimal opcode reference
- `../../../docs/PLATFORM.md` – common machine architecture and cost/value rationale


## Bidirectional linear memory walking
Use `[Rn+]` for forward traversal and `[-Rn]` for backward traversal. The latter decrements by 1 for byte accesses and by 2 for word accesses before touching memory. This gives compact overlap-safe `memmove` loops without a special block-copy instruction.

## Fast zero page

For compiler-oriented static data, `ZLOAD8/ZLOAD16/ZSTORE8/ZSTORE16` address `0x00..0xFF` in two bytes and implicitly use R0. Generated SVM-C code reserves `0x00..0xEF` for fast statics and starts code at `0x0100`.

## Timer / interrupt quick reference

The shared machine provides a 32-bit virtual clock, one 16-bit timer, and timer/VSYNC/keyboard IRQ sources at `0xFF12..0xFF1F`. Configure the vector and source mask while interrupts are disabled, acknowledge handled bits through `IRQ_ACK` (`0xFF14`), then return with `IRET`. See the project-level `../../../docs/PLATFORM.md` for the normative MMIO semantics.


## Register ISA v3 code-density change

The `B0..BF` one-byte compact family encodes `AND` for `R0..R3`; `XOR` remains a fully supported general two-register instruction. The hardware `SUBI` immediate family is retained because `ADDI -imm16` preserves the numeric result but does not preserve the observable carry/no-borrow flag semantics in every case. Reallocating only the compact logical slot still gives the desired masking code-density benefit.

## Graphics library

`graphics.asm` exports the fast `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, and `vline` primitives plus the higher-level `line`, `rect`, `fillrect`, `circle`, and `fillcircle` procedures. ABI: R0=color; palette: R0..R3; putpixel: R0=x,R1=y; clear: R0=color; hline: R0=x0,R1=x1,R2=y; vline: R0=x,R1=y0,R2=y1. Unused procedures are removed by procedure-GC.

Shapes with five or more logical parameters use the same 16-bit graphics parameter block on every ISA: `GFX_X0=0x00C0`, `GFX_Y0=0x00C2`, `GFX_X1=0x00C4`, `GFX_Y1=0x00C6`, `GFX_W=0x00C8`, `GFX_H=0x00CA`, `GFX_R=0x00CC`, `GFX_COLOR=0x00CE`. `0x00B0..0x00BE` is internal virtual-register scratch on some targets; `0x00D0..0x00FA` is additional graphics scratch/current-colour storage. Therefore `graphics.asm` reserves the full `0x00B0..0x00FA` range. `line` reads `(x0,y0,x1,y1,color)`, `rect/fillrect` read `(x,y,w,h,color)`, and `circle/fillcircle` read `(cx,cy,r,color)`. Procedure-GC removes unused shape procedures and dependencies.


## Typed arithmetic reference include

The Register standard library also contains `typed_arith.asm` and `typed_convert.asm` as an educational typed arithmetic/conversion reference. See the common `TYPED_ARITHMETIC_LIBRARY_HU.md` / `TYPED_ARITHMETIC_LIBRARY_EN.md` documentation. The portable full IEEE soft-float implementation remains in SVM-C.
