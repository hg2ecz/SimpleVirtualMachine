# Memory-register Assembly Programming Manual


> Current video model: system memory and video memory are separate 16-bit spaces. Ordinary memory instructions never cross into video memory. See `../../../docs/PLATFORM.md` for the authoritative map and the architecture-specific video instruction forms.

The Memory-register CPU is a PIC-inspired, cost-oriented 16-bit architecture without historical banked-memory restrictions. Arithmetic centers on W and direct zero-page file operands. Two FSR registers provide full 64 KiB indirect access.

Programs normally reserve `0x0000..0x00EF` for fast variables and compiler data, `0x00F0..0x00FF` for compiler/scratch use, and load code at `0x0100` or above.

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
