# SVM ISA family – current reference

This document is the current common reference index for the nine implemented CPU architectures. Detailed opcode tables live in `svm_asm/docs/<cpu>/INSTRUCTION_REFERENCE_EN.md`; normative specifications for the newer ISA families live in `docs/*_ISA_SPEC_HU.md` where no separate English normative file exists yet.

| Target | Operand model | General data state | Main cost/value decision |
|---|---|---|---|
| `stack` | zero-address stack | data stack + lazy `TOS/NOS` cache + minimal `C` | assembly/Forth operations retained; `UMUL`, `ADC/SBC/RCR1`; two-cell stack cache |
| `accumulator` | one-address | `A` + `X/Y` address registers | implicit-A operations and integer carry assists |
| `memreg` | working register + file | `W`, file, `FSR0/1` | hot `ADD/AND`; compiler scratch at `0x000E..0x000F` |
| `register` | register-register | `R0..R7` | compact `AND` on `R0..R3`; native `SUBI` for correct flags |
| `loadstore` | strict three-address load/store | `R0..R7` | separate RAM/VRAM load-store; native long `SUBI16` |
| `regmem` | two-address register-memory | `R0..R7` | descriptor-based register/memory/immediate source, few opcode families |
| `memory2memory` | memory-to-memory | memory + `A0..A3` address registers | descriptor-based memory operands and direct RMW |
| `belt` | implicit-result belt | latest results `b0..b7` | every value-producing operation creates a new `b0` |
| `tta` | transport-triggered | `R0..R7` + functional-unit ports | moving data to a trigger port starts the operation |

## Common integer minimum

Where the operand model naturally supports them, the ISAs provide `AND/OR/XOR/NOT`, 16-bit add/subtract, multiply/divide/modulo, shifts, and low-cost multiword assists equivalent to `ADC/SBC/MULHU/RCR1`. The Stack architecture expresses the same capability with minimal carry state and `UMUL ( a b -- lo hi )`.

There is no hardware `f16/f32`, 32-bit ALU, or CLZ instruction. Floating-point arithmetic is provided by software libraries.

## Current encoding decisions

### Register

- `B0..BF`: compact `AND Rd,Rs` for `R0..R3`.
- `XOR` remains a normal full ALU operation.
- `D0..D7`: native `SUBI Rd,imm16`; it is not replaced by `ADDI -imm`, because carry/no-borrow semantics would differ.

### MemReg

- `C0..CF`: hot `ADD f,W`.
- `D0..DF`: hot `ADD f,F`.
- `E0..EF`: hot `AND f,W`.
- `F0..FF`: hot `AND f,F`.
- `XOR` remains available in the normal ALU form.

### Load/Store

`SUBI` uses the dedicated long-immediate major `9`, function `3` decode (`SUBI16`) to preserve the correct `C = no-borrow` semantics. The architecture deliberately has no auto-increment load/store.

## Assembly-oriented instructions

The `assembly-oriented` label does **not** mean deprecated or optional. Stack instructions such as `NIP/TUCK/2DUP/2DROP`, `PICK/ROLL`, and `DO/?DO/I/J/LOOP/+LOOP/LEAVE/UNLOOP` remain supported primarily for readable and dense hand-written stack/Forth assembly.

## Belt16 and TTA16

The normative Belt16 description is `docs/BELT_ISA_SPEC_HU.md`; its assembly documentation is under `svm_asm/docs/belt/`.

The normative TTA16 description is `docs/TTA_ISA_SPEC_HU.md`; its assembly documentation is under `svm_asm/docs/tta/`. TTA16 operations are triggered by explicit transports to ALU and memory ports; there is no direct core `ADD Rd,Rs` instruction.

## Stack microarchitecture

The programmer-visible Stack ISA is unchanged, but the reference runtime/hardware model uses a two-cell lazy `TOS/NOS` stack cache. It reduces data-stack RAM traffic without introducing new opcodes. Design rationale: `docs/ARCHITECTURE_DESIGN_RATIONALE_HU.md`.
