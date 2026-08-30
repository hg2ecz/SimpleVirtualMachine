# Common SVM platform

All nine CPUs use the same machine environment. Only the instruction set, register/stack model, and execution organization differ.

## CPU address space

- 16-bit addressing: `0x0000..0xFFFF`.
- `0x0000..0xFEFF` is contiguous RAM.
- MMIO occupies the top 256-byte page `0xFF00..0xFFFF`; currently defined registers are in `0xFF00..0xFF2A`.
- There is no guest-visible System ROM.
- The former System ROM area is gone: `0xE000..0xFEFF` is ordinary RAM, while `0xFF00..0xFFFF` is MMIO.
- Instruction fetch occurs only from the CPU address space, never from VRAM.

## Video RAM

VRAM is a separate 16 KiB data address space (`0x0000..0x3FFF`). It is never used for instruction fetch.

- 320x200 pixels, 2 bpp;
- framebuffer: 16,000 bytes (`0x0000..0x3E7F`);
- remaining 384 bytes (`0x3E80..0x3FFF`) are reserved;
- four simultaneous pixel colour slots; each slot selects one entry from a fixed 16-colour master palette.

## Character generator

The 8x8 ASCII font is a 768-byte internal character ROM inside the shared video device. It is not part of the CPU address space. Writing `TEXT_CHAR` (`0xFF06`) causes the video device to expand the selected glyph directly into the framebuffer using the configured foreground/background slots. No CPU-specific firmware/font ROM is required.

## Console

- `0xFF20`: VT100/RS232 `DATA`
- `0xFF21`: `STATUS`

SVM-C `putc/getc/puts` generate direct MMIO access rather than firmware calls. Assembly code can use the same registers directly or the include-able console helper libraries.

## Implementation consequence

The platform is implemented by one shared memory/video device model; the nine CPU cores contain only their architecture-specific execution logic.

## Hardware-assisted random generator

The shared platform contains a small hardware-style PRNG peripheral. It is MMIO rather than a CPU instruction, so all nine architectures see the same interface and peripheral cost.

- `0xFF26`: `RNG_DATA_LO` — reading produces and latches a new 16-bit sample;
- `0xFF27`: `RNG_DATA_HI` — high byte of the last latched sample;
- `0xFF28`: `RNG_STATUS` — bit 0 (`RNG_READY`) is currently always 1;
- `0xFF29..0xFF2A`: writable 16-bit `RNG_SEED`.

A normal 16-bit read at `load16(0xFF26)` returns one coherent sample. The reference VM uses deterministic `xorshift32`, which is useful for reproducible tests but is **not** a true entropy source and is not cryptographically secure. A physical implementation may connect the same MMIO interface to noise/jitter entropy; an emulator may use host-OS entropy when nondeterminism is desired.

The C library interface is in `hrandom.sc`: `hrand()`, `hrand_max()`, `hrand_seed()`, and `hrand_range()`.

## Integer arithmetic assists

Multiword integer and soft-float libraries use small low-cost integer assist instructions where the operand model naturally supports them. There is no hardware floating point. See `ARCHITECTURE_DESIGN_RATIONALE_HU.md` for the design rationale.

### Belt16 internal stack areas

RAM is physically contiguous; it is not split into hardware stack islands. By runtime convention, the top 1 KiB of RAM (`0xFB00..0xFEFF`) is reserved for the active CPU stack area. Stack, Belt16, and TTA16 split this into two 512-byte regions: data stack `0xFB00..0xFCFF`, control/return stack `0xFD00..0xFEFF`. These are still ordinary RAM addresses, not separate address spaces.

### Stack-pointer convention

On Load/Store and Register-Memory, `R6` is the architecturally visible stack pointer. On Memory-to-Memory, `A3` is the stack pointer. Compiler temporary `PUSH/POP`, `CALL/RET`, and IRQ save/restore all use the same stack; there is no hidden second SP.
