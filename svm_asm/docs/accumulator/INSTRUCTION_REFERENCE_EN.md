# Accumulator CPU instruction reference

The CPU state is deliberately small: 16-bit `A`, 16-bit `X`, low-cost 16-bit address register `Y`, 16-bit `PC`, 16-bit `SP`, and `Z/N/C` flags. Multi-byte operands are little-endian.

| Hex | Instruction | Bytes | Meaning |
|---:|---|---:|---|
| `00` | `NOP` | 1 | No operation |
| `01` | `HALT` | 1 | Stop CPU |
| `02` | `RET` | 1 | Pop return address into PC |
| `03` | `TAX` | 1 | `X=A` |
| `04` | `TXA` | 1 | `A=X` |
| `05` | `PUSHA` | 1 | Push A |
| `06` | `POPA` | 1 | Pop A |
| `07` | `PUSHX` | 1 | Push X |
| `08` | `POPX` | 1 | Pop X |
| `09` | `INC` | 1 | `A=A+1` |
| `0A` | `DEC` | 1 | `A=A-1` |
| `0B` | `NEG` | 1 | `A=0-A` |
| `0C` | `NOT` | 1 | `A=~A` |
| `0D` | `SHL1` | 1 | `A<<=1` |
| `0E` | `SHR1` | 1 | logical `A>>=1` |
| `0F` | `INX` | 1 | `X=X+1` |
| `10` | `DEX` | 1 | `X=X-1` |
| `11` | `ADDX` | 1 | `A=A+X` |
| `12` | `SUBX` | 1 | `A=A-X` |
| `13` | `MULX` | 1 | `A=A*X` |
| `14` | `DIVX` | 1 | unsigned `A=A/X` |
| `15` | `MODX` | 1 | unsigned `A=A%X` |
| `16` | `ANDX` | 1 | `A=A&X` |
| `17` | `ORX` | 1 | `A=A|X` |
| `18` | `XORX` | 1 | `A=A^X` |
| `19` | `SHLX` | 1 | `A <<= (X&15)` |
| `1A` | `SHRX` | 1 | `A >>= (X&15)` |
| `1B` | `CMPX` | 1 | flags from `A-X`, A unchanged |
| `1C` | `LDA8 [X]` | 1 | zero-extend byte at X into A |
| `1D` | `LDA16 [X]` | 1 | load word at X into A |
| `1E` | `STA8 [X]` | 1 | store low byte of A at X |
| `1F` | `STA16 [X]` | 1 | store A at X |
| `20` | `LDA8 [X+]` | 1 | load byte, then `X+=1` |
| `21` | `LDA16 [X+]` | 1 | load word, then `X+=2` |
| `22` | `STA8 [X+]` | 1 | store byte, then `X+=1` |
| `23` | `STA16 [X+]` | 1 | store word, then `X+=2` |
| `40` | `LDAI imm16` | 3 | `A=imm16` |
| `41` | `LDXI imm16` | 3 | `X=imm16` |
| `42` | `ADDI imm16` | 3 | `A+=imm16` |
| `43` | `SUBI imm16` | 3 | `A-=imm16` |
| `44` | `CMPI imm16` | 3 | flags from `A-imm16` |
| `45` | `ANDI imm16` | 3 | `A&=imm16` |
| `46` | `ORI imm16` | 3 | `A|=imm16` |
| `47` | `XORI imm16` | 3 | `A^=imm16` |
| `50` | `LDA8 addr16` | 3 | absolute byte load |
| `51` | `LDA16 addr16` | 3 | absolute word load |
| `52` | `STA8 addr16` | 3 | absolute byte store |
| `53` | `STA16 addr16` | 3 | absolute word store |
| `60` | `JMP addr16` | 3 | unconditional jump |
| `61` | `CALL addr16` | 3 | push return PC and jump |
| `62` | `JZ addr16` | 3 | jump if Z=1 |
| `63` | `JNZ addr16` | 3 | jump if Z=0 |
| `64` | `JC addr16` | 3 | jump if C=1 |
| `65` | `JNC addr16` | 3 | jump if C=0 |
| `66` | `JN addr16` | 3 | jump if N=1 |
| `67` | `JNN addr16` | 3 | jump if N=0 |

Opcodes `3A..3F`, `49..4F`, `54..5F`, and `70..FF` are reserved. `30..39` are assigned to zero-page, interrupt/DSP and video-extension operations. Free opcode space is intentionally not treated as a reason to add hardware.

Arithmetic updates Z/N and, where meaningful, C. `CMPX`/`CMPI` update flags without changing A.


## Encoding, length and execution-time quick reference

Times follow `../../../svm_rt/docs/CYCLE_MODEL.md`. Instruction fetch is charged byte-for-byte; byte data access is +1 cycle, word data access +2, and explicitly iterative arithmetic adds its internal cost. There is no taken-branch penalty.

| Assembly form | Hex encoding | Bytes | Cycles | Notes |
|---|---|---:|---:|---|
| `NOP`, `HALT`, register transfers/inc/dec/unary, `EI`, `DI`, `ASR1` | corresponding one-byte opcode | 1 | 1 | internal register operation |
| `RET`, `PUSHA/POPA`, `PUSHX/POPX` | `02`, `05..08` | 1 | 3 | one 16-bit stack access |
| `IRET` | `36` | 1 | 5 | two 16-bit stack reads |
| `ADDX/SUBX/ANDX/ORX/XORX/CMPX` | `11,12,16..18,1B` | 1 | 1 | register ALU |
| `MULX`, `DIVX`, `MODX` | `13..15` | 1 | 17 | 1 fetch + 16 internal |
| `SHLX`, `SHRX` | `19`, `1A` | 1 | 2 | +1 variable-shift internal cycle |
| byte load/store through X/Y | `1C,1E,20,22,28,2A,2C,2E` | 1 | 2 | +1 byte data access |
| word load/store through X/Y | `1D,1F,21,23,29,2B,2D,2F` | 1 | 3 | +2 word data access |
| zero-page byte load/store | `30 aa`, `32 aa` | 2 | 3 | 2 fetch + byte access |
| zero-page word load/store | `31 aa`, `33 aa` | 2 | 4 | 2 fetch + word access |
| `MULQ15X` | `38` | 1 | 18 | 1 fetch + 17 internal |
| video byte load/store | `39 ss` | 2 | 3 | video data space |
| video word load/store | `39 ss` | 2 | 4 | video data space |
| `LDAI/LDXI/LDYI/ADDI/SUBI/CMPI/ANDI/ORI/XORI imm16` | `40..48 lo hi` | 3 | 3 | immediate fetch |
| absolute byte load/store | `50/52 lo hi` | 3 | 4 | +1 data access |
| absolute word load/store | `51/53 lo hi` | 3 | 5 | +2 data access |
| long `JMP/Jcc addr16` | `60,62..67 lo hi` | 3 | 3 | taken/not-taken equal |
| long `CALL addr16` | `61 lo hi` | 3 | 5 | +16-bit stack write |
| short `RJMP/RJcc rel8` | `68,6A..6F dd` | 2 | 2 | assembler selects automatically |
| short `RCALL rel8` | `69 dd` | 2 | 4 | +16-bit stack write |

Post-increment and pre-decrement address updates do not add cycles beyond the memory access.

## Second address register and bidirectional memory walking

`Y` is a low-cost second address/index register, not a general ALU register. It exists primarily so source and destination pointers can remain resident during copies and buffer processing.

| Instruction | Opcode | Meaning |
|---|---:|---|
| `TAY` | `24` | `Y=A` |
| `TYA` | `25` | `A=Y` |
| `INY` | `26` | `Y=Y+1` |
| `DEY` | `27` | `Y=Y-1` |
| `STA8 [Y]` | `28` | `mem8[Y]=A` |
| `STA16 [Y]` | `29` | `mem16[Y]=A` |
| `STA8 [Y+]` | `2A` | store, then `Y=Y+1` |
| `STA16 [Y+]` | `2B` | store, then `Y=Y+2` |
| `LDA8 [-X]` | `2C` | `X=X-1; A=mem8[X]` |
| `LDA16 [-X]` | `2D` | `X=X-2; A=mem16[X]` |
| `STA8 [-Y]` | `2E` | `Y=Y-1; mem8[Y]=A` |
| `STA16 [-Y]` | `2F` | `Y=Y-2; mem16[Y]=A` |
| `LDYI imm16` | `48` | `Y=imm16` |


## Short control transfers and relaxation

The source mnemonics remain `JMP`, `CALL`, `JZ`, `JNZ`, `JC`, `JNC`, `JN`, and `JNN`. The assembler automatically selects the two-byte PC-relative encoding when the destination is within signed 8-bit displacement of the PC after the instruction; otherwise it emits the three-byte absolute form above. Source code therefore stays readable and does not need a separate short mnemonic.

| Hex | Internal short form | Bytes | Meaning |
|---:|---|---:|---|
| `68` | `RJMP rel8` | 2 | `PC = next_pc + sign_extend(rel8)` |
| `69` | `RCALL rel8` | 2 | push `next_pc`, then relative call |
| `6A` | `RJZ rel8` | 2 | relative jump if Z=1 |
| `6B` | `RJNZ rel8` | 2 | relative jump if Z=0 |
| `6C` | `RJC rel8` | 2 | relative jump if C=1 |
| `6D` | `RJNC rel8` | 2 | relative jump if C=0 |
| `6E` | `RJN rel8` | 2 | relative jump if N=1 |
| `6F` | `RJNN rel8` | 2 | relative jump if N=0 |

This is a code-density optimization only; the long absolute forms remain available automatically for distant targets. Accumulator executables using these opcodes use the `SVA\x06` format.

## Zero-page direct forms

| Hex | Mnemonic | Bytes |
|---|---|---:|
| 30 | `LDA8Z addr8` | 2 |
| 31 | `LDA16Z addr8` | 2 |
| 32 | `STA8Z addr8` | 2 |
| 33 | `STA16Z addr8` | 2 |

These avoid the third address byte of absolute addressing without adding a page-base register.

## Interrupt control

| Hex | Instruction | Bytes | Effect |
|---:|---|---:|---|
| `34` | `EI` | 1 | enable maskable interrupts globally |
| `35` | `DI` | 1 | disable maskable interrupts globally |
| `36` | `IRET` | 1 | restore saved status/control state and PC |

Interrupt entry reuses the existing 1 KiB hardware stack and clears interrupt enable until `IRET` restores the saved state.

## Integer DSP extension

| Instruction | Hex encoding | Meaning |
|---|---|---|
| `ASR1` | `37` | Arithmetic right shift of accumulator `A`. |
| `MULQ15X` | `38` | `A = q15(A * X)`. |

`MULQ15` uses signed 16-bit operands, a 32-bit intermediate, arithmetic `>>15`, and saturates the unique `0x8000 * 0x8000` overflow case to `0x7FFF`.

## Separate video-space extension

`0x39` is the video-memory extension prefix. The second byte selects an X/Y-addressed video operation. It is a single logical VM instruction even though its encoding is two bytes.

| Mnemonic | Hex |
|---|---|
| `VLDA8 [X]` / `VLDA16 [X]` | `39 00` / `39 01` |
| `VSTA8 [X]` / `VSTA16 [X]` | `39 02` / `39 03` |
| `VLDA8 [X+]` / `VLDA16 [X+]` | `39 04` / `39 05` |
| `VSTA8 [X+]` / `VSTA16 [X+]` | `39 06` / `39 07` |
| `VSTA8 [Y]` / `VSTA16 [Y]` | `39 08` / `39 09` |
| `VSTA8 [Y+]` / `VSTA16 [Y+]` | `39 0A` / `39 0B` |
| `VLDA8 [-X]` / `VLDA16 [-X]` | `39 0C` / `39 0D` |
| `VSTA8 [-Y]` / `VSTA16 [-Y]` | `39 0E` / `39 0F` |


## Multiword integer assists

`ADCX` (`3A`) computes `A = A + X + C`; `SBCX` (`3B`) computes `A = A - X - (1-C)`; `MULHUX` (`3C`) returns the upper 16 bits of unsigned `A*X`; `RCR1` (`3D`) rotates right through carry. `SHL1/SHR1` copy the shifted-out bit to C.
