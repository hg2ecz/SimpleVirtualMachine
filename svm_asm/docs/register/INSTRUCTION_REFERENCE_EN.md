# Register Machine Instruction Reference

This document is the normative programmer-facing instruction definition for the current cost-optimized register-machine ISA (`SVM\x09`). All multi-byte 16-bit immediates and addresses are encoded little-endian.


## Encoding, length and execution-time quick reference

The execution time below follows the VM cycle model in `../../../svm_rt/docs/CYCLE_MODEL.md`: every fetched instruction byte costs 1 cycle, an 8-bit data access costs 1 cycle, a 16-bit data access costs 2 cycles, and explicitly multi-cycle arithmetic adds its internal cost. There is no pipeline branch penalty. `C` means cycles.

| Assembly form | Hex encoding | Bytes | C | Notes |
|---|---|---:|---:|---|
| `NOP`, `HALT`, `EI`, `DI` | `00`, `01`, `07`, `08` | 1 | 1 | fetch only |
| `RET` | `02` | 1 | 3 | fetch + 16-bit stack read |
| `ZLOAD8 addr8` / `ZSTORE8 addr8` | `03 aa` / `05 aa` | 2 | 3 | 2 fetch + byte data access |
| `ZLOAD16 addr8` / `ZSTORE16 addr8` | `04 aa` / `06 aa` | 2 | 4 | 2 fetch + word data access |
| `IRET` | `09` | 1 | 5 | two 16-bit stack reads |
| `ASR1 Rr` | `0A rr` | 2 | 2 | second byte is register number |
| `MULQ15 Rd,Rs` | `0B 00dddsss` | 2 | 19 | 2 fetch + 17 internal |
| `VLOAD8/VSTORE8 ...` | `0C ss 00dddsss` | 3 | 4 | video byte access |
| `VLOAD16/VSTORE16 ...` | `0C ss 00dddsss` | 3 | 5 | video word access |
| unary `NOT/NEG/INC/DEC/SHL1/SHR1 Rr` | `10..3F` | 1 | 1 | register encoded in opcode |
| `PUSH Rr` / `POP Rr` | `40..4F` | 1 | 3 | 16-bit hardware-stack access |
| compact `MOV/ADD/SUB/CMP/AND Rd,Rs` (`R0..R3`) | `50..8F`, `B0..BF` | 1 | 1 | both registers in opcode |
| compact `LOAD8` / `STORE8` | `90..AF` | 1 | 2 | fetch + byte data access |
| `MOVI/ADDI/SUBI/CMPI Rr,imm16` | `C0..DF lo hi` | 3 | 3 | immediate fetch only |
| general `MOV/ADD/SUB/AND/OR/XOR/CMP` | `E0/E1/E2/E6/E7/E8/EB pp` | 2 | 2 | `pp=00dddsss` |
| `MUL/DIV/MOD Rd,Rs` | `E3/E4/E5 pp` | 2 | 18 | 2 fetch + 16 internal |
| `SHL/SHR Rd,Rs` | `E9/EA pp` | 2 | 3 | +1 internal variable-shift cost |
| general `LOAD8/STORE8` | `EC/EE pp` | 2 | 3 | byte data access |
| general `LOAD16/STORE16` | `ED/EF pp` | 2 | 4 | word data access |
| `JMP/Jcc addr16` | `F0,F2..F7 lo hi` | 3 | 3 | taken/not-taken cost is equal |
| `CALL addr16` | `F1 lo hi` | 3 | 5 | +16-bit return-address write |
| post-inc/pre-dec byte load/store | `F8/F9/FC/FD pp` | 2 | 3 | address update is internal |
| post-inc/pre-dec word load/store | `FA/FB/FE/FF pp` | 2 | 4 | address update is internal |

The video subopcode `ss` is `00..0B` as listed later in this reference. Invalid encodings trap before a successful instruction retires.

## CPU state

- Eight 16-bit general-purpose registers: `R0`..`R7`.
- `R0`..`R3` are also the compact register subset for selected one-byte two-register instructions.
- 16-bit program counter `PC`.
- 16-bit hardware stack pointer `SP`, used by `PUSH`, `POP`, `CALL`, and `RET`.
- Flags: `Z` (zero), `N` (negative/sign bit), `C` (carry/no-borrow).

## Encoding notation

- `r` / `rrr`: 3-bit register number (`0..7`).
- `d`, `s`: destination and source register numbers.
- `dd`, `ss`: 2-bit compact register numbers (`R0..R3`).
- `imm16`: 16-bit little-endian immediate.
- `addr16`: 16-bit little-endian absolute address.
- General register-pair byte: `00dddsss`.
- Compact two-register opcode: `base | (dd << 2) | ss`.

## Fixed instructions

| Mnemonic | Hex opcode | Bytes | Definition | Flags |
|---|---:|---:|---|---|
| `NOP` | `00` | 1 | No operation. | unchanged |
| `HALT` | `01` | 1 | Halt the CPU until reset/host intervention. | unchanged |
| `RET` | `02` | 1 | Pop a 16-bit return address from the hardware stack into `PC`. | unchanged |

`0D..0F` are currently invalid/reserved. Opcodes `03..0C` are assigned to zero-page, interrupt/DSP and video-extension functions described below.

## Embedded-register unary instructions

The lower three opcode bits select `R0..R7`. Each instruction is exactly one byte.

| Mnemonic | Opcode range | Formula | Definition | Flags |
|---|---|---|---|---|
| `NOT Rr` | `10..17` | `10 | r` | `Rr = ~Rr` | `Z,N` updated; `C` unchanged |
| `NEG Rr` | `18..1F` | `18 | r` | `Rr = 0 - Rr` | `Z,N` updated; `C=1` iff operand was zero |
| `INC Rr` | `20..27` | `20 | r` | `Rr = Rr + 1` | `Z,N,C` updated |
| `DEC Rr` | `28..2F` | `28 | r` | `Rr = Rr - 1` | `Z,N,C` updated (`C` = no borrow) |
| `SHL1 Rr` | `30..37` | `30 | r` | Logical left shift by one. | `Z,N` updated; `C` = old bit15 |
| `SHR1 Rr` | `38..3F` | `38 | r` | Logical right shift by one. | `Z,N` updated; `C` = old bit0 |
| `PUSH Rr` | `40..47` | `40 | r` | Push the 16-bit register value on the hardware stack. | unchanged |
| `POP Rr` | `48..4F` | `48 | r` | Pop a 16-bit value from the hardware stack into `Rr`. | unchanged |

Example: `INC R3` encodes as `23`.

## Compact two-register instructions (`R0..R3` only)

These forms are one byte. The opcode is `base | (dd << 2) | ss`, where `dd` and `ss` encode only `R0..R3`.

| Mnemonic | Hex range | Base | Definition | Flags |
|---|---:|---:|---|---|
| `MOV Rd, Rs` | `50..5F` | `50` | `Rd = Rs` | unchanged |
| `ADD Rd, Rs` | `60..6F` | `60` | `Rd = Rd + Rs` | `Z,N,C` updated |
| `SUB Rd, Rs` | `70..7F` | `70` | `Rd = Rd - Rs` | `Z,N,C` updated (`C` = no borrow) |
| `CMP Rd, Rs` | `80..8F` | `80` | Compute `Rd - Rs` for flags only. | `Z,N,C` updated |
| `LOAD8 Rd, [Rs]` | `90..9F` | `90` | `Rd = zero_extend(mem8[Rs])` | unchanged |
| `STORE8 [Rd], Rs` | `A0..AF` | `A0` | `mem8[Rd] = low8(Rs)` | unchanged |
| `AND Rd, Rs` | `B0..BF` | `B0` | `Rd = Rd AND Rs` | `Z,N` updated; `C` unchanged |

Example: `ADD R2, R1` = `60 | (2 << 2) | 1` = `69`.

The assembler automatically uses these encodings when both registers are in `R0..R3`; otherwise it uses the general form below. `XOR` remains fully supported in the general two-register family; `AND` receives the compact slot because masking is substantially more common in the wide-integer and soft-float libraries.

## Embedded-register immediate instructions

The lower three bits select `R0..R7`; two immediate bytes follow.

| Mnemonic | Opcode range | Formula | Bytes | Definition | Flags |
|---|---:|---|---:|---|---|
| `MOVI Rr, imm16` | `C0..C7` | `C0 | r` | 3 | `Rr = imm16` | unchanged |
| `ADDI Rr, imm16` | `C8..CF` | `C8 | r` | 3 | `Rr = Rr + imm16` | `Z,N,C` updated |
| `SUBI Rr, imm16` | `D0..D7` | `D0 | r` | 3 | `Rr = Rr - imm16` | `Z,N,C` updated (`C` = no borrow) |
| `CMPI Rr, imm16` | `D8..DF` | `D8 | r` | 3 | Compute `Rr - imm16` for flags only. | `Z,N,C` updated |

Assembler cost reductions include `ADDI Rn,1 -> INC Rn`, `SUBI Rn,1 -> DEC Rn`, and `MOV Rn,Rn -> NOP`. `SUBI` remains a hardware instruction because replacing it with `ADDI -imm` would preserve the numeric result but not the observable carry/no-borrow flag semantics in all cases.

## General two-register / memory instructions

These instructions use one opcode byte followed by a register-pair byte `00dddsss`; total length is two bytes. All `R0..R7` are available.

| Mnemonic | Hex opcode | Bytes | Pair meaning | Definition | Flags |
|---|---:|---:|---|---|---|
| `MOV Rd, Rs` | `E0` | 2 | `d=Rd,s=Rs` | `Rd = Rs` | unchanged |
| `ADD Rd, Rs` | `E1` | 2 | same | `Rd = Rd + Rs` | `Z,N,C` updated |
| `SUB Rd, Rs` | `E2` | 2 | same | `Rd = Rd - Rs` | `Z,N,C` updated |
| `MUL Rd, Rs` | `E3` | 2 | same | `Rd = Rd * Rs` (low 16 bits) | `Z,N` updated; `C=0` |
| `DIV Rd, Rs` | `E4` | 2 | same | Unsigned `Rd = Rd / Rs`; divide-by-zero traps. | `Z,N` updated |
| `MOD Rd, Rs` | `E5` | 2 | same | Unsigned `Rd = Rd % Rs`; divide-by-zero traps. | `Z,N` updated |
| `AND Rd, Rs` | `E6` | 2 | same | `Rd = Rd AND Rs` | `Z,N` updated |
| `OR Rd, Rs` | `E7` | 2 | same | `Rd = Rd OR Rs` | `Z,N` updated |
| `XOR Rd, Rs` | `E8` | 2 | same | `Rd = Rd XOR Rs` | `Z,N` updated |
| `SHL Rd, Rs` | `E9` | 2 | same | Logical left shift `Rd` by `(Rs & 15)`. | `Z,N` updated |
| `SHR Rd, Rs` | `EA` | 2 | same | Logical right shift `Rd` by `(Rs & 15)`. | `Z,N` updated |
| `CMP Rd, Rs` | `EB` | 2 | same | Compute `Rd - Rs` for flags only. | `Z,N,C` updated |
| `LOAD8 Rd, [Ra]` | `EC` | 2 | `d=Rd,s=Ra` | `Rd = zero_extend(mem8[Ra])` | unchanged |
| `LOAD16 Rd, [Ra]` | `ED` | 2 | `d=Rd,s=Ra` | `Rd = mem16[Ra]` | unchanged |
| `STORE8 [Ra], Rs` | `EE` | 2 | `d=Ra,s=Rs` | `mem8[Ra] = low8(Rs)` | unchanged |
| `STORE16 [Ra], Rs` | `EF` | 2 | `d=Ra,s=Rs` | `mem16[Ra] = Rs` | unchanged |

The register-pair byte must have its upper two bits clear; otherwise the VM reports an invalid register encoding.

## Absolute control-flow instructions

Each instruction is three bytes: opcode followed by `addr16` in little-endian order.

| Mnemonic | Hex opcode | Definition |
|---|---:|---|
| `JMP addr16` | `F0` | `PC = addr16` |
| `CALL addr16` | `F1` | Push the address after this instruction, then `PC = addr16`. |
| `JZ addr16` | `F2` | Jump if `Z=1`. |
| `JNZ addr16` | `F3` | Jump if `Z=0`. |
| `JC addr16` | `F4` | Jump if `C=1`. |
| `JNC addr16` | `F5` | Jump if `C=0`. |
| `JN addr16` | `F6` | Jump if `N=1`. |
| `JNN addr16` | `F7` | Jump if `N=0`. |

## Post-increment indirect memory instructions

Each is two bytes: opcode plus register-pair byte `00dddsss`. These are deliberately limited to the high-value linear-memory case; there is no general complex addressing-mode subsystem.

| Assembly syntax | Internal mnemonic | Hex | Pair meaning | Definition |
|---|---|---:|---|---|
| `LOAD8 Rd, [Ra+]` | `LOAD8P` | `F8` | `d=Rd,s=Ra` | `Rd = zero_extend(mem8[Ra]); Ra += 1` |
| `STORE8 [Ra+], Rs` | `STORE8P` | `F9` | `d=Ra,s=Rs` | `mem8[Ra] = low8(Rs); Ra += 1` |
| `LOAD16 Rd, [Ra+]` | `LOAD16P` | `FA` | `d=Rd,s=Ra` | `Rd = mem16[Ra]; Ra += 2` |
| `STORE16 [Ra+], Rs` | `STORE16P` | `FB` | `d=Ra,s=Rs` | `mem16[Ra] = Rs; Ra += 2` |

For post-increment loads, `Rd` and `Ra` must be different registers. Stores may use the same source/address register if desired.

`FC..FF` are the pre-decrement memory-walker family.

## Flag definitions

- `Z`: set when the result is zero.
- `N`: set when result bit 15 is one.
- `C` after addition: carry out of bit 15.
- `C` after subtraction/compare: **one means no borrow**, zero means borrow.

Loads, stores, moves, stack operations, branches, calls, and returns do not modify flags unless explicitly stated above.

## Endianness and arithmetic

- 16-bit memory values and instruction immediates are little-endian.
- Arithmetic wraps modulo 65536 unless an instruction explicitly traps (division/modulo by zero).
- Shifts are logical, not arithmetic.

## Pre-decrement memory walkers

These instructions complement the post-increment forms and make backward `memmove` loops cost-symmetric with forward loops. The address register is decremented before the access.

| Assembly | Opcode | Bytes | Semantics |
|---|---:|---:|---|
| `LOAD8 Rd,[-Ra]` | `FC` | 2 | `Ra=Ra-1; Rd=mem8[Ra]` |
| `STORE8 [-Ra],Rs` | `FD` | 2 | `Ra=Ra-1; mem8[Ra]=Rs` |
| `LOAD16 Rd,[-Ra]` | `FE` | 2 | `Ra=Ra-2; Rd=mem16[Ra]` |
| `STORE16 [-Ra],Rs` | `FF` | 2 | `Ra=Ra-2; mem16[Ra]=Rs` |

For loads, data and address registers must be distinct.

## Zero-page compiler forms

| Hex | Instruction | Bytes | Meaning |
|---|---|---:|---|
| 03 | `ZLOAD8 addr8` | 2 | `R0 = mem8[addr8]` |
| 04 | `ZLOAD16 addr8` | 2 | `R0 = mem16[addr8]` |
| 05 | `ZSTORE8 addr8` | 2 | `mem8[addr8] = R0.low` |
| 06 | `ZSTORE16 addr8` | 2 | `mem16[addr8] = R0` |

R0 is implicit deliberately: it is already the SVM-C expression/result register, so a general-register zero-page encoding would cost substantially more opcode space for little recurring benefit.

## Interrupt control

| Hex | Instruction | Bytes | Effect |
|---:|---|---:|---|
| `07` | `EI` | 1 | set global interrupt-enable state |
| `08` | `DI` | 1 | clear global interrupt-enable state |
| `09` | `IRET` | 1 | restore saved status/control state and PC from the hardware stack |

Interrupt entry saves PC and status, clears interrupt enable, and jumps to the MMIO-configured IRQ vector. Pending sources are acknowledged through `IRQ_ACK`; `IRET` does not acknowledge a source.

## Integer DSP extension

| Instruction | Hex encoding | Meaning |
|---|---|---|
| `ASR1 Rn` | `0A rr` | Arithmetic right shift of `Rn` by one bit. |
| `MULQ15 Rd,Rs` | `0B 00dddsss` | Signed Q15 multiply; result is written to `Rd`. |

`MULQ15` uses signed 16-bit operands, a 32-bit intermediate, arithmetic `>>15`, and saturates the unique `0x8000 * 0x8000` overflow case to `0x7FFF`.

## Separate video-space extension

Video memory is a separate 16-bit data-only address space. The register ISA uses the three-byte form `0C ss pp`, where `ss` is the video-memory subopcode and `pp=00dddsss` is the ordinary register-pair byte. These operations never access system memory and instruction fetch never uses video space.

| Mnemonic | Hex | Semantics |
|---|---|---|
| `VLOAD8 Rd,[Ra]` | `0C 00 pp` | `Rd = video8[Ra]` |
| `VLOAD16 Rd,[Ra]` | `0C 01 pp` | `Rd = video16[Ra]` |
| `VSTORE8 [Ra],Rs` | `0C 02 pp` | `video8[Ra] = Rs` |
| `VSTORE16 [Ra],Rs` | `0C 03 pp` | `video16[Ra] = Rs` |
| `VLOAD8P Rd,[Ra+]` | `0C 04 pp` | load byte, then `Ra += 1` |
| `VLOAD16P Rd,[Ra+]` | `0C 05 pp` | load word, then `Ra += 2` |
| `VSTORE8P [Ra+],Rs` | `0C 06 pp` | store byte, then `Ra += 1` |
| `VSTORE16P [Ra+],Rs` | `0C 07 pp` | store word, then `Ra += 2` |
| `VLOAD8M Rd,[-Ra]` | `0C 08 pp` | `Ra -= 1`, then load |
| `VLOAD16M Rd,[-Ra]` | `0C 09 pp` | `Ra -= 2`, then load |
| `VSTORE8M [-Ra],Rs` | `0C 0A pp` | `Ra -= 1`, then store |
| `VSTORE16M [-Ra],Rs` | `0C 0B pp` | `Ra -= 2`, then store |


## Multiword integer assists

| Instruction | Encoding | Effect |
|---|---|---|
| `ADC Rd,Rs` | `0D 00 00dddsss` | `Rd = Rd + Rs + C`, C=carry-out |
| `SBC Rd,Rs` | `0D 01 00dddsss` | `Rd = Rd - Rs - (1-C)`, C=1 means no borrow |
| `MULHU Rd,Rs` | `0D 02 00dddsss` | upper 16 bits of unsigned `Rd*Rs` |
| `RCR1 Rd` | `0D 03 rr` | rotate right through carry |

`SHL1` writes the old bit15 to C and `SHR1` writes the old bit0 to C. There is no hardware floating point.
