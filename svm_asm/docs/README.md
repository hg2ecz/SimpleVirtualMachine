# Assembler documentation

This directory contains documentation specific to `svm-asm` and handwritten assembly programming.

## Common assembler documentation

- [`ASSEMBLER_REFERENCE_HU.md`](ASSEMBLER_REFERENCE_HU.md) - command line, targets, output formats, memory conventions and reusable libraries.
- [`SOURCE_INCLUDES_HU.md`](SOURCE_INCLUDES_HU.md) - `.include`, `-I`, include-once, recursion and cycle handling.
- [`CONSOLE_LIBRARY_HU.md`](CONSOLE_LIBRARY_HU.md) - character console helpers.
- [`GRAPHICS_LIBRARY_HU.md`](GRAPHICS_LIBRARY_HU.md) - 2-bpp graphics helpers and palette-slot model.
- [`TEXT_SCREEN_LIBRARY_HU.md`](TEXT_SCREEN_LIBRARY_HU.md) - 40x25 framebuffer text-layer helpers.

## Per-architecture manuals

Each architecture directory contains an assembly programming manual and instruction reference in Hungarian and English. The two language versions are maintained as semantic pairs: opcode sets, encodings, cycle rules and ISA limitations must match even when examples or prose differ:

- `register/`
- `stack/`
- `accumulator/`
- `memreg/`
- `loadstore/`
- `regmem/`
- `memory2memory/`
- `belt/`
- `tta/`

Common ISA and platform specifications are not duplicated here; see [`../../docs/README.md`](../../docs/README.md).
