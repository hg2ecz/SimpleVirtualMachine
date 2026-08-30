# Stack Machine Instruction Reference

> **Notation:** “assembly-oriented” means that the main reason for retaining the instruction is hand-written stack/Forth-style assembly programmability; the C backend does not require it. The instruction remains a full, stable ISA primitive.


This document is the normative programmer-facing instruction definition for the current cost-optimized stack-machine ISA (`SVS\x08`). The machine uses 16-bit cells, a two-cell TOS/NOS lazy stack cache, a data stack, and a shared return/control stack. Multi-byte 16-bit values are little-endian.


## Encoding, length and execution-time quick reference

Stack timings require one explicit convention because the machine has a cached TOS. The cycle model is in `../../../svm_rt/docs/CYCLE_MODEL.md`.

- instruction-byte fetch: 1 cycle per byte;
- 8-bit data access: +1 cycle;
- 16-bit data/stack access: +2 cycles;
- TOS and NOS register accesses cost no extra VM cycle;
- pushing spills one 16-bit cached cell only when both TOS and NOS are already occupied: `S=2`;
- a binary operation refills NOS from RAM only when NOS is invalid and a second operand is needed: `N=2`;
- popping refills the new TOS from RAM only when NOS is invalid and a RAM-backed item remains: `R=2`;
- binary operations leave NOS invalid instead of eagerly refilling it;
- `MUL/DIV/MOD`: +16 internal cycles; `MULQ15`: +17; variable `SHL/SHR`: +1.

Therefore some timings are cache-state dependent. `S`, `N` and `R` below denote the real RAM access only when it occurs.

| Assembly form | Hex | Bytes | Cycles | Notes |
|---|---:|---:|---:|---|
| `NOP`, `HALT` | `00`,`01` | 1 | 1 | fetch only |
| `RET` | `02` | 1 | 3 | return-stack read |
| `DUP` | `03` | 1 | `1+S` = 1 or 3 | spills only when both TOS and NOS are occupied |
| `DROP` | `04` | 1 | `1` or `3` | +R when another item remains |
| `SWAP` | `05` | 1 | `1+N` = 1 or 3 | register swap when NOS is cached |
| `OVER` | `06` | 1 | 3 | either reads second item or spills one cached cell |
| `ROT` | `07` | 1 | 5 or 7 | third-cell read/write; +N if NOS was lazy |
| `NIP` | `08` | 1 | `1+N` = 1 or 3 | cache-state change on NOS hit |
| `TUCK` | `09` | 1 | 3 or 5 | one stack-word spill; +N if NOS was lazy |
| `2DUP` | `0A` | 1 | 5 | two resulting cache overflows/reads total two word accesses |
| `2DROP` | `0B` | 1 | 1 or 3 | at most one refill of the new TOS |
| `C@+`, `C@-` | `0C`,`3C` | 1 | `2+S` = 2 or 4 | byte read; pushed value may spill one cached cell |
| `@+`, `@-` | `0E`,`3E` | 1 | `3+S` = 3 or 5 | word read; pushed value may spill one cached cell |
| `C!+`, `C!-` | `0D`,`3D` | 1 | `2+N` = 2 or 4 | byte write; value comes from NOS |
| `!+`, `!-` | `0F`,`3F` | 1 | `3+N` = 3 or 5 | word write; value comes from NOS |
| binary ALU/compare `+ - AND OR XOR = ...` | `10,11,18..1A,20..25` | 1 | `1+N` = 1 or 3 | register-only when NOS is valid; result leaves NOS lazy |
| `MUL/DIV/MOD` | `12..14` | 1 | 17 or 19 | +16 internal, plus optional lazy NOS refill |
| unary `NEG,1+,1-,NOT,2*,2/,0=,0<` | `15..17,1B,1E,1F,26,27` | 1 | 1 | TOS-only |
| variable `SHL/SHR` | `1C`,`1D` | 1 | 2 or 4 | +1 internal, plus optional lazy NOS refill |
| `C@` | `28` | 1 | 2 | address already in TOS |
| `@` | `29` | 1 | 3 | word data read |
| `C!` | `2A` | 1 | `2+N+R` = 2,4,6 | value from NOS plus optional refill of new TOS |
| `!` | `2B` | 1 | `3+N+R` = 3,5,7 | value from NOS plus optional refill of new TOS |
| one-byte literal `-1,0..10` | `30..3B` | 1 | `1+S` = 1 or 3 | spill only if both cache cells are occupied |
| `PUSH8/PUSHS8` | `40/41 ii` | 2 | `2+S` = 2 or 4 | literal fetch + optional spill |
| `BRA8` | `42 dd` | 2 | 2 | no pipeline penalty |
| `BZ8/BNZ8` | `43/44 dd` | 2 | 2 or 4 | +R if popped flag reveals backed item |
| `CALL8` | `45 dd` | 2 | 4 | return-stack write |
| `PICK depth` | `4A dd` | 2 | depth 0: `2+S`; depth>0: `4+S` | selected backed item read |
| `ROLL depth` | `4B dd` | 2 | depth-dependent | multiple backed reads/writes; see formula below |
| zero-page byte load | `4C aa` | 2 | `3+S` | byte read + push |
| zero-page word load | `4D aa` | 2 | `4+S` | word read + push |
| zero-page byte store | `4E aa` | 2 | 3 or 5 | +R after consuming value if backed item remains |
| zero-page word store | `4F aa` | 2 | 4 or 6 | +R after consuming value if backed item remains |
| `SYS EI/DI` | `50 00/01` | 2 | 2 | prefix + subopcode |
| `SYS IRET` | `50 02` | 2 | 6 | two return-stack reads |
| `SYS ASR1` | `50 03` | 2 | 2 | TOS-only |
| `SYS MULQ15` | `50 04` | 2 | 19 or 21 | +17 internal plus optional NOS refill |
| video `VC@/VC!` | `50 10/12` | 2 | analogous to `C@/C!` +1 prefix byte | separate VRAM |
| video `V@/V!` | `50 11/13` | 2 | analogous to `@/!` +1 prefix byte | separate VRAM |
| `PUSH16 imm16` | `80 lo hi` | 3 | `3+S` = 3 or 5 | optional spill |
| `JMP addr16` | `81 lo hi` | 3 | 3 | absolute |
| `JZ/JNZ addr16` | `82/83 lo hi` | 3 | 3 or 5 | possible refill after flag pop |
| `CALL addr16` | `84 lo hi` | 3 | 5 | return-stack write |
| absolute byte load | `89 lo hi` | 3 | `4+S` | data read + push |
| absolute word load | `8A lo hi` | 3 | `5+S` | word read + push |
| absolute byte store | `8B lo hi` | 3 | 4 or 6 | possible refill |
| absolute word store | `8C lo hi` | 3 | 5 or 7 | possible refill |

Loop instructions use the shared return/control stack and therefore expose their real frame-access cost:

| Loop instruction | Short/long bytes | Cycles | Path dependence |
|---|---:|---:|---|
| `DO` | 1 | 9 or 11 | +2 if data remains below the two consumed parameters |
| `I`, `J` | 1 | `5+S` | two return-stack word reads, then push index |
| `UNLOOP` | 1 | 1 | pointer update only |
| `?DO8` | 2 | equal: 6 or 8; enter: 10 or 12 | entering writes two loop-frame cells |
| `?DO` | 3 | equal: 7 or 9; enter: 11 or 13 | same as above plus one address byte |
| `LOOP8` | 2 | exit: 6; continue: 8 | continue writes updated index |
| `LOOP` | 3 | exit: 7; continue: 9 | absolute target has one extra fetch byte |
| `+LOOP8` | 2 | exit: `6+R`; continue: `8+R` | step is popped from data stack |
| `+LOOP` | 3 | exit: `7+R`; continue: `9+R` | same, absolute form |
| `LEAVE8` / `LEAVE` | 2 / 3 | 2 / 3 | loop-frame removal is a pointer update |

`ROLL n` is intentionally depth-dependent because it moves backed cells rather than hiding that cost behind a fixed nominal timing.

## Stack notation

Stack effects are written as:

`( before -- after )`

The rightmost item is the top of the data stack. Boolean true is `0xFFFF`; false is `0x0000`.

## Instruction length rule

The opcode's top two bits encode the total instruction length:

- `00xxxxxx` -> 1 byte
- `01xxxxxx` -> 2 bytes
- `10xxxxxx` -> 3 bytes
- `11xxxxxx` -> 4 bytes (currently unused/reserved)

This allows cheap instruction-length decoding.

## One-byte core and stack instructions

| Mnemonic | Hex | Stack effect | Definition |
|---|---:|---|---|
| `NOP` | `00` | `( -- )` | No operation. |
| `HALT` | `01` | `( -- )` | Halt the CPU. |
| `RET` / `EXIT` | `02` | `( -- )` | Pop return address from the return/control stack into `PC`. The assembler inserts required `UNLOOP`s before an `EXIT` from active structured loops. |
| `DUP` | `03` | `( a -- a a )` | Duplicate TOS. |
| `DROP` | `04` | `( a -- )` | Drop TOS. |
| `SWAP` | `05` | `( a b -- b a )` | Exchange top two cells. |
| `OVER` | `06` | `( a b -- a b a )` | Copy second cell to TOS. |
| `ROT` | `07` | `( a b c -- b c a )` | Rotate top three cells. |
| `NIP` | `08` | `( a b -- b )` | Remove second cell; **assembly-oriented convenience**. |
| `TUCK` | `09` | `( a b -- b a b )` | Copy TOS beneath second cell; **assembly-oriented convenience**. |
| `2DUP` | `0A` | `( a b -- a b a b )` | Duplicate top pair; **assembly-oriented convenience**. |
| `2DROP` | `0B` | `( a b -- )` | Drop top pair; **assembly-oriented convenience**. |

## One-byte post-increment memory walkers

These are cost-optimized linear-memory primitives. They combine memory access and address advancement without a separate `INC`/`2 +` sequence.

| Mnemonic | Aliases | Hex | Stack effect | Definition |
|---|---|---:|---|---|
| `C@+` | `LOAD8+` | `0C` | `( addr -- addr+1 value )` | Read unsigned byte, keep advanced address. |
| `C!+` | `STORE8+` | `0D` | `( value addr -- addr+1 )` | Store low byte, keep advanced address. |
| `@+` | `LOAD16+` | `0E` | `( addr -- addr+2 value )` | Read 16-bit cell, keep advanced address. |
| `!+` | `STORE16+` | `0F` | `( value addr -- addr+2 )` | Store 16-bit cell, keep advanced address. |

## One-byte arithmetic and bit operations

| Mnemonic | Hex | Stack effect | Definition |
|---|---:|---|---|
| `+` / `ADD` | `10` | `( a b -- a+b )` | 16-bit wrapping addition. |
| `-` / `SUB` | `11` | `( a b -- a-b )` | 16-bit wrapping subtraction. |
| `*` / `MUL` | `12` | `( a b -- a*b )` | Low 16 bits of product. |
| `/` / `DIV` | `13` | `( a b -- a/b )` | Unsigned division; divide-by-zero traps. |
| `MOD` | `14` | `( a b -- a%b )` | Unsigned remainder; divide-by-zero traps. |
| `NEGATE` / `NEG` | `15` | `( a -- -a )` | Two's-complement negation. |
| `1+` / `INC` | `16` | `( a -- a+1 )` | Increment. |
| `1-` / `DEC` | `17` | `( a -- a-1 )` | Decrement. |
| `AND` | `18` | `( a b -- a&b )` | Bitwise AND. |
| `OR` | `19` | `( a b -- a|b )` | Bitwise OR. |
| `XOR` | `1A` | `( a b -- a^b )` | Bitwise XOR. |
| `NOT` | `1B` | `( a -- ~a )` | Bitwise NOT. |
| `LSHIFT` / `SHL` | `1C` | `( value count -- result )` | Logical left shift by `count & 15`. |
| `RSHIFT` / `SHR` | `1D` | `( value count -- result )` | Logical right shift by `count & 15`. |
| `2*` / `SHL1` | `1E` | `( a -- a<<1 )` | One-bit logical left shift. |
| `2/` / `SHR1` | `1F` | `( a -- a>>1 )` | One-bit logical right shift. |

## One-byte comparisons

All comparisons produce `0xFFFF` for true and `0x0000` for false.

| Mnemonic | Hex | Stack effect | Definition |
|---|---:|---|---|
| `=` / `EQ` | `20` | `( a b -- flag )` | True if `a == b`. |
| `<>` / `NE` | `21` | `( a b -- flag )` | True if `a != b`. |
| `U<` / `ULT` | `22` | `( a b -- flag )` | Unsigned `a < b`. |
| `U>` / `UGT` | `23` | `( a b -- flag )` | Unsigned `a > b`. |
| `<` / `SLT` | `24` | `( a b -- flag )` | Signed 16-bit `a < b`. |
| `>` / `SGT` | `25` | `( a b -- flag )` | Signed 16-bit `a > b`. |
| `0=` | `26` | `( a -- flag )` | True if zero. |
| `0<` | `27` | `( a -- flag )` | True if signed value is negative. |

## One-byte memory and loop-frame instructions

| Mnemonic | Hex | Stack effect | Definition |
|---|---:|---|---|
| `C@` / `LOAD8` | `28` | `( addr -- value )` | Read unsigned byte and replace address with value. |
| `@` / `LOAD16` | `29` | `( addr -- value )` | Read 16-bit cell. |
| `C!` / `STORE8` | `2A` | `( value addr -- )` | Store low byte. |
| `!` / `STORE16` | `2B` | `( value addr -- )` | Store 16-bit cell. |
| `DO` | `2C` | `( limit start -- )` | Push a two-cell loop frame `(limit,index=start)` on the shared return/control stack. |
| `I` | `2D` | `( -- index )` | Push current loop index. |
| `J` | `2E` | `( -- outer-index )` | Push enclosing loop's index. |
| `UNLOOP` | `2F` | `( -- )` | Remove current loop frame from the return/control stack. |

## Dense one-byte literal window

These opcodes push a literal without an immediate byte. The assembler selects them automatically.

| Source literal | Hex | Stack effect |
|---:|---:|---|
| `-1` / `TRUE` | `30` | `( -- FFFF )` |
| `0` | `31` | `( -- 0000 )` |
| `1` | `32` | `( -- 0001 )` |
| `2` | `33` | `( -- 0002 )` |
| `3` | `34` | `( -- 0003 )` |
| `4` | `35` | `( -- 0004 )` |
| `5` | `36` | `( -- 0005 )` |
| `6` | `37` | `( -- 0006 )` |
| `7` | `38` | `( -- 0007 )` |
| `8` | `39` | `( -- 0008 )` |
| `9` | `3A` | `( -- 0009 )` |
| `10` | `3B` | `( -- 000A )` |

## Two-byte instructions

The second byte is either an immediate, a signed relative offset, or a depth parameter.

| Mnemonic | Hex | Operand byte | Stack effect | Definition |
|---|---:|---|---|---|
| `PUSH8 u8` | `40` | unsigned 8-bit literal | `( -- value )` | Zero-extend literal to 16 bits. |
| `PUSHS8 s8` | `41` | signed 8-bit literal | `( -- value )` | Sign-extend literal to 16 bits. |
| `BRA8 rel8` | `42` | signed PC-relative offset | `( -- )` | Unconditional relative branch. |
| `BZ8 rel8` | `43` | signed PC-relative offset | `( flag -- )` | Branch if popped value is zero. |
| `BNZ8 rel8` | `44` | signed PC-relative offset | `( flag -- )` | Branch if popped value is nonzero. |
| `CALL8 rel8` | `45` | signed PC-relative offset | `( -- )` | Push return address on control stack and branch relative. |
| `?DO8 rel8` | `46` | signed PC-relative offset | `( limit start -- )` | If start==limit, branch to loop exit; otherwise create loop frame. |
| `LOOP8 rel8` | `47` | signed PC-relative offset | `( -- )` | Increment loop index by 1; branch while loop continues, otherwise remove frame. |
| `+LOOP8 rel8` | `48` | signed PC-relative offset | `( step -- )` | Advance loop index by signed step; branch while loop continues. |
| `LEAVE8 rel8` | `49` | signed PC-relative offset | `( -- )` | Remove current loop frame and branch to loop exit. |
| `PICK depth` | `4A` | unsigned depth | `( ... x ... -- ... x ... x )` | Copy stack item at depth 0=TOS to TOS. |
| `ROLL depth` | `4B` | unsigned depth | varies | Move item at the given depth to TOS, shifting intervening items. |

`4C..4F` are zero-page direct memory forms and `50` is the system prefix, as described below. `51..7F` are currently unassigned/reserved two-byte opcode space.

Relative offsets are measured from the `PC` after the offset byte has been fetched. The assembler automatically chooses short relative forms when the target fits.

## Three-byte instructions

The opcode is followed by a 16-bit little-endian immediate or absolute address.

| Mnemonic | Hex | Stack effect | Definition |
|---|---:|---|---|
| `PUSH16 imm16` | `80` | `( -- value )` | Push 16-bit literal. |
| `JMP addr16` | `81` | `( -- )` | Absolute jump. |
| `JZ addr16` | `82` | `( flag -- )` | Pop flag; jump if zero. |
| `JNZ addr16` | `83` | `( flag -- )` | Pop flag; jump if nonzero. |
| `CALL addr16` | `84` | `( -- )` | Push return address and jump absolute. |
| `?DO addr16` | `85` | `( limit start -- )` | Zero-trip loop setup; jump to exit if `start == limit`. |
| `LOOP addr16` | `86` | `( -- )` | Increment current loop index and jump while continuing. |
| `+LOOP addr16` | `87` | `( step -- )` | Advance current loop by signed step and jump while continuing. |
| `LEAVE addr16` | `88` | `( -- )` | Remove current loop frame and jump to exit. |
| `LOAD8ABS addr16` *(assembler-generated)* | `89` | `( -- value )` | Read unsigned byte from absolute address and push it. |
| `LOAD16ABS addr16` *(assembler-generated)* | `8A` | `( -- value )` | Read 16-bit cell from absolute address and push it. |
| `STORE8ABS addr16` *(assembler-generated)* | `8B` | `( value -- )` | Store low byte to absolute address. |
| `STORE16ABS addr16` *(assembler-generated)* | `8C` | `( value -- )` | Store 16-bit cell to absolute address. |

`8D..BF` are currently unassigned/reserved three-byte opcode space. `C0..FF` (four-byte class) is currently entirely reserved.

## Structured-control assembler behavior

The source assembler performs cost-oriented encoding choices automatically:

- `-1` and `0..10` use one-byte literal opcodes.
- Other fitting positive 8-bit literals use `PUSH8`; fitting signed negatives use `PUSHS8`; remaining 16-bit values use `PUSH16`.
- Local branches/calls/loop transfers use the 8-bit relative form when the final displacement fits; otherwise the absolute 16-bit form is emitted.
- Constant-address patterns may be folded into the internal `LOAD8ABS`, `LOAD16ABS`, `STORE8ABS`, or `STORE16ABS` machine forms instead of pushing the address and executing the generic memory primitive.
- `EXIT` inside structured `DO` nesting is compiled with the required `UNLOOP` operations before `RET`.

## Arithmetic and memory rules

- Cells are 16-bit and arithmetic wraps modulo 65536.
- `DIV` and `MOD` are unsigned and trap on division by zero.
- `SLT`, `SGT`, and `0<` interpret values as signed two's-complement 16-bit integers.
- `SHL`/`SHR` mask the shift count with 15.
- 16-bit memory accesses use little-endian byte order.

## Pre-decrement memory walkers

The four one-byte opcodes formerly used for the rarely important dedicated literals 11..14 are used for backward linear memory walking. Literals 11..14 still assemble normally through `PUSH8`.

| Word | Opcode | Stack effect |
|---|---:|---|
| `C@-` / `LOAD8-` | `3C` | `( addr -- addr-1 value )` |
| `C!-` / `STORE8-` | `3D` | `( value addr -- addr-1 )` |
| `@-` / `LOAD16-` | `3E` | `( addr -- addr-2 value )` |
| `!-` / `STORE16-` | `3F` | `( value addr -- addr-2 )` |

## Return/control stack note

`DO...LOOP` frames share the return/control stack with call return addresses. `I` and `J` are intended for the word that owns the active loop. A called word must not assume that the caller's loop index is directly accessible through `I/J`. This explicit restriction avoids the cost of a third loop stack or a dedicated loop-frame pointer.

## Zero-page direct forms

`0x4C..0x4F` are two-byte `Load8Zp`, `Load16Zp`, `Store8Zp`, `Store16Zp` instructions (opcode + address8). The assembler normally selects them automatically when a constant address followed by `C@`, `@`, `C!`, or `!` is in `0x00..0xFF`.

## Interrupt control system prefix

IRQ control deliberately uses the free two-byte system prefix `50 xx` rather than consuming another hot one-byte opcode.

| Bytes | Instruction | Effect |
|---|---|---|
| `50 00` | `EI` | enable maskable interrupts globally |
| `50 01` | `DI` | disable maskable interrupts globally |
| `50 02` | `IRET` | restore saved interrupt-enable state and PC from the return/control stack |

The assembler accepts the plain `EI`, `DI`, and `IRET` mnemonics. Interrupt entry uses the existing return/control stack; no third interrupt stack is added.

## Integer DSP extension

| Instruction | Hex encoding | Stack effect |
|---|---|---|
| `ASR1` | `50 03` | `( x -- x/2 )`, arithmetic signed shift. |
| `MULQ15` | `50 04` | `( a b -- q15(a*b) )`. |

`MULQ15` uses signed 16-bit operands, a 32-bit intermediate, arithmetic `>>15`, and saturates the unique `0x8000 * 0x8000` overflow case to `0x7FFF`.

## Separate video-space system extensions

The stack machine preserves its dense one-byte opcode space by encoding video-memory operations as `SYS` (`0x50`) plus a subopcode. Video addresses are 16-bit offsets in the separate video data space.

| Mnemonic | Hex | Stack effect |
|---|---|---|
| `VC@` | `50 10` | `( addr -- value )` |
| `V@` | `50 11` | `( addr -- value )` |
| `VC!` | `50 12` | `( value addr -- )` |
| `V!` | `50 13` | `( value addr -- )` |
| `VC@+` | `50 14` | `( addr -- addr+1 value )` |
| `V@+` | `50 15` | `( addr -- addr+2 value )` |
| `VC!+` | `50 16` | `( value addr -- addr+1 )` |
| `V!+` | `50 17` | `( value addr -- addr+2 )` |
| `VC@-` | `50 18` | `( addr -- addr-1 value )` |
| `V@-` | `50 19` | `( addr -- addr-2 value )` |
| `VC!-` | `50 1A` | `( value addr -- addr-1 )` |
| `V!-` | `50 1B` | `( value addr -- addr-2 )` |


## Multiword integer assist

The Stack ISA has one minimal `C` carry/borrow state for multiword arithmetic; comparisons still produce stack values and there is no general status-register model.

- `ADD` (`10`): `( a b -- r )`, `C` = carry-out.
- `SUB` (`11`): `( a b -- r )`, `C=1` means no borrow.
- `SHL1` (`1E`): `( a -- r )`, old bit15 -> `C`.
- `SHR1` (`1F`): `( a -- r )`, old bit0 -> `C`.
- `ADC` (`50 06`): `( a b -- r )`, `r=a+b+C`, updates `C`.
- `SBC` (`50 07`): `( a b -- r )`, `r=a-b-(1-C)`, updates no-borrow `C`.
- `RCR1` (`50 08`): `( a -- r )`, old `C` -> bit15, old bit0 -> `C`.
- `UMUL` / `MUL32` (`50 05`): `( a b -- lo hi )`, full unsigned 16x16 -> 32-bit product.
