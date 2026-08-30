# Memory-register CPU Instruction Reference


## Encoding, length and execution-time quick reference

Times follow `../../../svm_rt/docs/CYCLE_MODEL.md`. Every fetched byte costs 1 cycle; byte data access costs 1, word access 2, and the iterative arithmetic operations add their stated internal cost.

| Assembly form | Hex encoding | Bytes | Cycles | Notes |
|---|---|---:|---:|---|
| `NOP`, `HALT`, W unary ops, W/FSR transfers, `EI`, `DI`, `ASR1W` | `00,01,05..0F,19,1A` | 1 | 1 | register/internal |
| `RET`, `PUSHW`, `POPW` | `02..04` | 1 | 3 | one 16-bit stack access |
| `IRET` | `1B` | 1 | 5 | two stack reads |
| immediate `LDI/FSR0I/FSR1I/ADDI/...` | `10..18 lo hi` | 3 | 3 | immediate fetch |
| direct `MOV8 f,W` / `MOV8 W,f` | `20/21 ff` | 2 | 3 | byte file access |
| direct `MOV16 f,W` / `MOV16 W,f` | `22/23 ff` | 2 | 4 | word file access |
| direct ALU `... f,W`, `CMP f` | even/direct W forms | 2 | 4 | word read |
| direct ALU `... f,F`, `INC f`, `DEC f` | write-back forms | 2 | 6 | word read + word write |
| FSR byte load/store, any hold/+/- mode | `30,32,34,36,38,3A,3C,3E,40,42,44,46` | 1 | 2 | byte access |
| FSR word load/store, any hold/+/- mode | `31,33,35,37,39,3B,3D,3F,41,43,45,47` | 1 | 3 | word access |
| `SHL f,W`, `SHR f,W` | `48/49 ff` | 2 | 5 | word read +1 internal |
| `MUL/DIV/MOD f,W` | `4A/4B/4C ff` | 2 | 20 | word read +16 internal |
| `MULQ15 f,W` | `4D ff` | 2 | 21 | word read +17 internal |
| long `JMP/Jcc addr16` | `60,62..67 lo hi` | 3 | 3 | no branch penalty |
| long `CALL addr16` | `61 lo hi` | 3 | 5 | +stack write |
| short `RJMP/RJcc rel8` | `68,6A..6F dd` | 2 | 2 | assembler relaxation |
| short `RCALL rel8` | `69 dd` | 2 | 4 | +stack write |
| hot `MOV8` | `80..9F` | 1 | 2 | file address in low nibble |
| hot `MOV16` | `A0..BF` | 1 | 3 | file address in low nibble |
| hot `ADD/AND f,W` | `C0..CF`, `E0..EF` | 1 | 3 | word read |
| hot `ADD/AND f,F` | `D0..DF`, `F0..FF` | 1 | 5 | word read + write |
| video-space byte operation | `1C ss` | 2 | 3 | +video byte access |
| video-space word operation | `1C ss` | 2 | 4 | +video word access |

The low nibble of a hot-file opcode is the file address `0x00..0x0F`. FSR pre-decrement/post-increment itself has no extra cycle charge.

## Programmer-visible state

- `W`: 16-bit working/accumulator register.
- `FSR0`, `FSR1`: 16-bit indirect address registers.
- `PC`, `SP`.
- flags: `Z`, `N`, `C`.
- direct file space: `0x00..0xFF` (zero page).
- hot file space: `0x00..0x0F`; selected operations encode the address in the opcode and are one byte.

`d=W` means the result is written to W. `d=F` means the result is written back to the file operand.

## Fixed one-byte instructions

| Hex | Mnemonic | Meaning |
|---|---|---|
| 00 | NOP | no operation |
| 01 | HALT | halt |
| 02 | RET | return |
| 03 / 04 | PUSHW / POPW | hardware stack |
| 05 / 06 | INCW / DECW | W +/- 1 |
| 07 / 08 | NEGW / NOTW | unary W |
| 09 / 0A | SHL1W / SHR1W | W shift by one |
| 0B / 0C | W2F0 / W2F1 | W -> FSR0/1 |
| 0D / 0E | F02W / F12W | FSR0/1 -> W |

## Immediate instructions (3 bytes)

| Hex | Mnemonic |
|---|---|
| 10 | LDI imm16 |
| 11 | FSR0I imm16 |
| 12 | FSR1I imm16 |
| 13 | ADDI imm16 |
| 14 | SUBI imm16 |
| 15 | CMPI imm16 |
| 16 | ANDI imm16 |
| 17 | ORI imm16 |
| 18 | XORI imm16 |

## Direct file instructions (2 bytes)

The second byte is the zero-page file address.

| Hex | Form | Semantics |
|---|---|---|
| 20 | MOV8 f,W | W = mem8[f] |
| 21 | MOV8 W,f | mem8[f] = W.low |
| 22 | MOV16 f,W | W = mem16[f] |
| 23 | MOV16 W,f | mem16[f] = W |
| 24/25 | ADD f,W / ADD f,F | W=W+F / F=F+W |
| 26/27 | SUB f,W / SUB f,F | W=W-F / F=F-W |
| 28/29 | AND f,W / AND f,F | bitwise AND |
| 2A/2B | OR f,W / OR f,F | bitwise OR |
| 2C/2D | XOR f,W / XOR f,F | bitwise XOR |
| 2E | CMP f | flags from W-F |
| 2F | INC f | increment 16-bit file |
| 48 | SHL f,W | W <<= (F & 15) |
| 49 | SHR f,W | W >>= (F & 15) |
| 4A | MUL f,W | W *= F |
| 4B | DIV f,W | W /= F |
| 4C | MOD f,W | W %= F |
| 4F | DEC f | decrement 16-bit file |

## Indirect memory walkers (1 byte)

FSR0 opcodes are `30..3B`, FSR1 opcodes are `3C..47`. Each FSR supports byte/word load/store in three modes: unchanged, post-increment, pre-decrement.

Examples: `LDB0`, `LDW0+`, `STB1+`, `LDW0-`, `STW1-`.

Post-increment changes the FSR after the access by 1 (byte) or 2 (word). Pre-decrement changes it before the access.

## Control transfer

Long absolute forms `60..67`: `JMP CALL JZ JNZ JC JNC JN JNN` (3 bytes).
Short relative forms `68..6F`: same order, 2 bytes. The assembler automatically relaxes a source-level branch/call to the short form when the signed 8-bit displacement fits.

## One-byte hot file encodings

For file addresses `0x00..0x0F`, the low opcode nibble is the file address:

| Range | Operation |
|---|---|
| 80..8F | MOV8 f,W |
| 90..9F | MOV8 W,f |
| A0..AF | MOV16 f,W |
| B0..BF | MOV16 W,f |
| C0..CF | ADD f,W |
| D0..DF | ADD f,F |
| E0..EF | AND f,W |
| F0..FF | AND f,F |

This is deliberately limited to high-frequency operations; unused opcode space is not filled merely because it exists. XOR remains available in the normal file/working-register ALU forms; AND receives the hot encoding because masking dominates the current wide-integer and soft-float workloads.

## Interrupt control

| Hex | Instruction | Bytes | Effect |
|---:|---|---:|---|
| `19` | `EI` | 1 | enable maskable interrupts globally |
| `1A` | `DI` | 1 | disable maskable interrupts globally |
| `1B` | `IRET` | 1 | restore saved status/control state and PC |

These opcodes use previously free space and add only one global enable state bit to the CPU.

## Integer DSP extension

| Instruction | Hex encoding | Meaning |
|---|---|---|
| `ASR1W` | `0F` | Arithmetic right shift of `W`. |
| `MULQ15 f,W` | `4D ff` | `W = q15(W * mem[ff])`. |

`MULQ15` uses signed 16-bit operands, a 32-bit intermediate, arithmetic `>>15`, and saturates the unique `0x8000 * 0x8000` overflow case to `0x7FFF`.

## Separate video-space extension

`0x1C` prefixes an indirect FSR operation in video space. Subcodes `00..0B` are FSR0 hold/post-increment/pre-decrement byte/word load/store forms; `0C..17` are the corresponding FSR1 forms.

Examples:

| Mnemonic | Hex | Meaning |
|---|---|---|
| `VLDB0` | `1C 00` | `video8[FSR0] -> W` |
| `VLDW0` | `1C 01` | `video16[FSR0] -> W` |
| `VSTB0+` | `1C 06` | `W -> video8[FSR0++]` |
| `VSTW0+` | `1C 07` | `W -> video16[FSR0]; FSR0+=2` |
| `VLDB0-` | `1C 08` | `--FSR0; video8[FSR0] -> W` |
| `VSTB0-` | `1C 0A` | `--FSR0; W -> video8[FSR0]` |
| `VLDB1` | `1C 0C` | `video8[FSR1] -> W` |
| `VSTB1+` | `1C 12` | `W -> video8[FSR1++]` |
| `VLDB1-` | `1C 14` | `--FSR1; video8[FSR1] -> W` |
| `VSTB1-` | `1C 16` | `--FSR1; W -> video8[FSR1]` |


## Multiword integer assists

Opcodes `50..55` provide `ADC f,W`, `ADC f,F`, `SBC f,W`, `SBC f,F`, `MULHU f,W`, and `RCR1W`. `ADC/SBC` propagate C; `MULHU` returns the upper word of unsigned 16x16 multiplication. `SHL1W/SHR1W` write the shifted-out bit to C.
