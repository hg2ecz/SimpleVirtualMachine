# SVM-S Stack Machine – Assembly Programming Manual


> Current video model: system memory and video memory are separate 16-bit spaces. Ordinary memory instructions never cross into video memory. See `../../../docs/PLATFORM.md` for the authoritative map and the architecture-specific video instruction forms.

This document is the programming manual for the current cost-optimized **SVM-S v2 / executable v3** stack machine. The assembler uses a Forth-like syntax, but the system is not an interactive Forth: source code is assembled ahead of time into machine code.

## 1. Programmer's model

The machine:

- operates on 16-bit cells;
- has a 16-bit address space;
- is little-endian;
- sees 64 KiB of memory;
- has no general-purpose register file;
- uses two physical stacks: data stack and a shared return/control stack;
- can cache the top two data-stack cells in internal 16-bit TOS/NOS registers; NOS is refilled lazily.

Programming is based on consciously tracking stack effects.

For example:

```text
ADD   ( a b -- a+b )
```

The rightmost item is the top of the stack.

## 2. Minimal program

```forth
.load 0x0200
.entry main

: main
    1 2 + DROP
    HALT
;
```

`:` starts a new definition and `;` emits `RET`.

## 3. Lexical rules

- source is tokenized by whitespace;
- names are case-insensitive;
- `\` starts a comment extending to the end of the line;
- decimal and `0x` hexadecimal numbers are accepted;
- `_` separators may be used inside numbers;
- an unknown word is assembled as a call and must resolve to a definition or label.

## 4. Literals and code density

A bare number automatically receives the shortest available encoding.

- `-1` / `TRUE`, and `0..10`: 1 byte
- remaining positive `0..255` values: `PUSH8`, 2 bytes
- `-128..-2`: `PUSHS8`, 2 bytes
- all other 16-bit values: `PUSH16`, 3 bytes

Example:

```forth
0 1 2 14 255 0xFF06
```

The assembler selects the physical encoding automatically.

`FALSE` = `0`, `TRUE` = `0xFFFF`.

## 5. The data stack

Basic operations:

```text
DUP      ( a -- a a )
DROP     ( a -- )
SWAP     ( a b -- b a )
OVER     ( a b -- a b a )
ROT      ( a b c -- b c a )
NIP      ( a b -- b )
TUCK     ( a b -- b a b )
2DUP     ( a b -- a b a b )
2DROP    ( a b -- )
PICK n
ROLL n

> **Assembly-oriented instructions:** `NIP`, `TUCK`, `2DUP`, `2DROP`, `PICK`, and `ROLL` remain real ISA instructions primarily because they improve density and readability of hand-written stack/Forth-style assembly. The C backend does not require them. This is a deliberate cost/value choice, not a compiler requirement.
```

The most important rule of stack programming is that each word should have a clear, preferably documented stack effect.

```forth
: square   \ ( n -- n2 )
    DUP *
;
```

## 6. Meaning of the TOS+NOS lazy stack cache

The logical top item resides in the internal `TOS` register. The second item can reside in the internal `NOS` register when it is actually needed. Neither register is directly addressable by software. `NOS` uses **lazy refill**: after a binary operation the CPU does not automatically fetch the next RAM-backed stack cell; a later instruction refills NOS only if it really needs a second operand.

Consequences:

- unary ALU operations require no data-stack RAM access;
- when both TOS and NOS are valid, binary ALU operations are register-only;
- `SWAP` is a register exchange on a cache hit;
- `NIP` can be a cache-state change only;
- `DUP` does not spill until both cache cells are occupied;
- a third and later pushed cell spills one 16-bit cached cell to stack RAM as needed;
- a binary result stays in TOS and the next NOS remains lazy until demanded.

This is a microarchitectural optimization: the ISA and stack effects do not change. The VM cycle counter charges only RAM accesses that actually occur.

## 7. Arithmetic

```text
+    ADD     ( a b -- a+b )
-    SUB     ( a b -- a-b )
*    MUL     ( a b -- low16(a*b) )
/    DIV     unsigned
MOD          unsigned remainder
NEG NEGATE   ( a -- -a )
1+  INC
1-  DEC
```

Operations are 16-bit and wrap modulo 65536.

Division by zero is a runtime error.

## 8. Logical and shift operations

```text
AND OR XOR NOT
SHL SHR
SHL1 / 2*
SHR1 / 2/
```

Binary `SHL/SHR` use the top stack item as the shift count with `count & 15` semantics.

For a single-bit shift, `2*`/`2/` gives the smallest code.

## 9. Comparison and Boolean values

```text
=  <>  U<  U>  <  >  0=  0<
```

Explicit aliases:

```text
EQ NE ULT UGT SLT SGT
```

The result is always a canonical Forth Boolean:

```text
false = 0x0000
true  = 0xFFFF
```

## 10. Basic memory access

```text
C@      ( addr -- value )
@       ( addr -- value )
C!      ( value addr -- )
!       ( value addr -- )
```

Aliases:

```text
LOAD8 LOAD16 STORE8 STORE16
```

`C@` zero-extends a byte. `C!` writes the low 8 bits of the value.

The 16-bit `@` and `!` forms are little-endian. A 16-bit access at `0xFFFF` is invalid.

## 11. Automatic absolute-memory optimization

If a memory operation is immediately preceded by a constant address, the assembler may emit a shorter absolute form.

Source:

```forth
65 0xFF06 C!
```

The programmer does not need to select a separate `STORE8ABS` mnemonic. The address is not pushed onto the data stack unnecessarily.

Therefore MMIO can be written naturally:

```forth
15 0xFF04 C!
```

## 12. Post-increment linear-memory primitives

The cost-optimized ISA contains four 1-byte primitives for linear memory traversal:

```text
C@+   ( addr -- addr+1 value )
C!+   ( value addr -- addr+1 )
@+    ( addr -- addr+2 value )
!+    ( value addr -- addr+2 )
```

Aliases:

```text
LOAD8+ STORE8+ LOAD16+ STORE16+
```

These are useful when the updated pointer must remain available after the access.

### Example: byte copy with two pointers

```forth
.load 0x0200
.entry main

: main
    0x3000 0x4000
    256 0 DO
        SWAP C@+ ROT C!+
    LOOP
    2DROP
    HALT
;
```

After the loop, the updated source and destination pointers remain on the stack; `2DROP` removes them.

## 13. Conditional structures

### IF / ELSE / THEN

```forth
condition IF
    ...
ELSE
    ...
THEN
```

`IF` consumes the condition.

Example:

```forth
: abs16   \ ( n -- |n| )
    DUP 0< IF NEG THEN
;
```

## 14. BEGIN loops

Infinite loop:

```forth
BEGIN
    ...
AGAIN
```

Conditional exit:

```forth
BEGIN
    ... condition
UNTIL
```

`UNTIL` exits when the flag is true.

WHILE form:

```forth
BEGIN
    ... condition
WHILE
    ...
REPEAT
```

## 15. DO / LOOP

> The `DO/?DO/I/J/LOOP/+LOOP/LEAVE/UNLOOP` family is part of the ISA primarily for **hand-written assembly/Forth-style programmability**. The C compiler can generate ordinary branch-based loops, so this block is not a compiler requirement; it is retained for the natural programming model and compact manual code of the stack architecture.

The parameter order is Forth-like:

```text
( limit start -- )
```

Example:

```forth
10 0 DO
    I DROP
LOOP
```

`I` pushes the current loop index, and `J` pushes the index of the next outer loop.

Because loop frames share the return/control stack with call return addresses, `I` and `J` are intended for the word that owns the active `DO...LOOP`. A called word must not assume that its caller's loop frame is directly accessible through `I/J`. This is an explicit cost-oriented restriction that avoids a third loop stack or loop-frame pointer.

### ?DO

```forth
10 10 ?DO
    ...
LOOP
```

If `start == limit`, the body is skipped entirely.

### +LOOP

```forth
10 0 DO
    I DROP
    2
+LOOP
```

The step is consumed from the data stack. Positive and negative steps are supported.

### LEAVE

Immediately exits the current counted loop.

### UNLOOP

Removes the innermost loop frame from the shared return/control stack. When `EXIT` is used inside active `DO` loops, the structured assembler automatically emits the required `UNLOOP` operations before `RET`; hand-written low-level control flow must preserve the same rule.

## 16. CASE

```forth
value CASE
    1 OF
        ...
    ENDOF
    2 OF
        ...
    ENDOF
    ... default ...
ENDCASE
```

## 17. Definitions and calls

```forth
: add-one   \ ( n -- n+1 )
    1+
;

: main
    41 add-one DROP
    HALT
;
```

`;` emits `RET`. `EXIT` also returns. `RECURSE` calls the current definition.

The CPU has a separate return stack, so data-stack values and return addresses do not mix.

## 18. Branch relaxation

The structured assembler first tries to use 8-bit relative branches. If a target does not fit in `-128..127`, it automatically expands the branch to a 16-bit absolute form.

The programmer therefore normally does not need to choose short versus long branch encodings.

This is an important part of code-size optimization.

## 19. Memory map

| Range / address | Function |
|---|---|
| `0x0000..0xFAFF` | program/data RAM |
| `0xFB00..0xFCFF` | data stack |
| `0xFD00..0xFEFF` | return/control stack |
| `0xFF00..0xFF01` | keyboard |
| `0xFF02..0xFF06` | text X/Y, FG/BG and `TEXT_CHAR` |
| `0xFF0B` | VSYNC counter |
| `0xFF0C..0xFF0F` | four 4-bit selectors into the fixed 16-colour master palette |
| separate video space `0x0000..0x3E7F` | 16,000-byte framebuffer |
| separate video space `0x3E80..0x3FFF` | 384 reserved bytes |

## 20. Video: 320x200x2 bpp, single VRAM

There is one 16 KiB video RAM and no bank/swap logic. `VC@/VC!`, `V@/V!` and their +/- walkers address video data only. Pixel values 0..3 select one of four slots; `0xFF0C..0xFF0F` map those slots to the fixed 16-colour master palette.

## 21. Internal character ROM and character generator

The text grid is 40x25. The glyph ROM is internal to the video device and is not CPU-addressable. The normal MMIO form remains available:

```forth
5  0xFF02 C!
4  0xFF03 C!
3  0xFF04 C!      \ foreground slot
0  0xFF05 C!      \ background slot
65 0xFF06 C!
```

Text output needs no firmware service. Write the character byte to `0xFF06`; cursor home is `0 0xFF02 C! 0 0xFF03 C!`.

## 22. Keyboard polling

```forth
: wait-key   \ ( -- key )
    BEGIN
        0xFF00 C@ 0=
    UNTIL
    0xFF01 C@
;
```

In low-level code, keep the stack effect of every structured loop path correct.

## 23. Host character output

The stack implementation also contains host-side character-output MMIO:

```text
0xFF20 CONSOLE_DATA
0xFF21 CONSOLE_STATUS
```

Example:

```forth
72 0xFF20 C!      \ H
73 0xFF20 C!      \ I
10 0xFF20 C!      \ newline
```

This is not the same as the `TEXT_CHAR` (`0xFF06`) character generator. `TEXT_CHAR` renders into the framebuffer, while `CONSOLE_DATA` writes to the reference runtime's VT100/RS232 terminal output.

## 24. Cost-optimized stack-programming rules

1. Keep stack effects short and unambiguous.
2. Use small literals naturally; the assembler chooses the shortest encoding.
3. For constant MMIO addresses, place the address directly before the memory operation so absolute-address optimization can apply.
4. Prefer `C@+`, `C!+`, `@+`, and `!+` for linear pointer traversal.
5. Let the assembler perform branch relaxation for simple local control flow.
6. Avoid unnecessary `SWAP/ROT`; design the stack layout of the word correctly from the beginning.
7. Repeated stack sequences may be factored into a colon definition when the call overhead is justified.
8. Do not use the data or shared return/control stack regions as ordinary RAM.

## 25. Common mistakes

- data-stack underflow during a binary operation;
- mismatch between a word's documented and actual stack effect;
- reversing the operands of `C!`: `( value addr -- )`;
- reversing the operands of `DO`: `( limit start -- )`;
- incorrect manual `EXIT`/`RET` handling while a loop frame is active;
- using `@` at address `0xFFFF`;
- confusing `TEXT_CHAR` with `CONSOLE_DATA`;
- treating `TEXT_FG/TEXT_BG` as master-palette indices instead of framebuffer slots.

## 26. Complete example: draw text using the single framebuffer

Set the text MMIO registers and write the character byte to `TEXT_CHAR`. No video-bank selection or firmware call is needed.

## 27. Related documents

- `../README.md` – assembler/ISA entry point
- `INSTRUCTION_REFERENCE_EN.md` – authoritative hexadecimal opcode reference
- `../../../docs/PLATFORM.md` – common machine architecture and cost/value rationale


## Bidirectional linear memory walking
Forward walkers are `C@+ C!+ @+ !+`; backward walkers are `C@- C!- @- !-`. Backward pointers start one element past the region and are decremented before the access. Dedicated one-byte literals remain for -1 and 0..10; 11..14 use the normal two-byte literal encoding.

## Fast zero page

Constant zero-page memory expressions such as `0x20 @` and `0x21 C!` are automatically encoded as two-byte direct operations. No source-level special syntax or page register is required.

## Timer / interrupt quick reference

The shared machine provides a 32-bit virtual clock, one 16-bit timer, and timer/VSYNC/keyboard IRQ sources at `0xFF12..0xFF1F`. Configure the vector and source mask while interrupts are disabled, acknowledge handled bits through `IRQ_ACK` (`0xFF14`), then return with `IRET`. See the project-level `../../../docs/PLATFORM.md` for the normative MMIO semantics.

## Minimal carry state

The Stack CPU keeps one hidden `C` bit solely for multiword integer arithmetic. `ADD` and `SHL1` write carry-out, `SUB` writes no-borrow, and `SHR1` writes the shifted-out bit0. `ADC`, `SBC`, and `RCR1` consume this state. Comparisons and conditional control still use explicit stack values; there is no general status register.
