# svm-asm

`svm-asm` is the assembler for all nine SimpleVirtualMachine ISA targets:
`register`, `stack`, `accumulator`, `memreg`, `loadstore`, `regmem`, `memory2memory`, `belt`, and `tta`.

## Usage

```sh
svm-asm register input.asm [output.svm]
svm-asm stack input.fsasm [output.svs]
svm-asm accumulator input.asm [output.sva]
svm-asm memreg input.fasm [output.svf]
svm-asm loadstore input.asm [output.svl]
svm-asm regmem input.asm [output.svr]
svm-asm memory2memory input.asm [output.svc]
svm-asm belt input.asm [output.svb]
svm-asm tta input.asm [output.svt]
```

## Documentation

Architecture-specific assembly programming manuals and instruction references are under [`docs/`](docs/). Common command-line and library reference: [`docs/ASSEMBLER_REFERENCE_HU.md`](docs/ASSEMBLER_REFERENCE_HU.md). The index is [`docs/README.md`](docs/README.md).

Common platform and ISA design documentation is kept at the repository level under [`../docs/`](../docs/).

## Source includes

Assembly sources can include reusable files without a macro preprocessor:

```asm
.include "lib/io.asm"
```

Relative files are searched first beside the including source and then in `-I` directories. Recursive includes, include-once behavior, cycle detection, and a maximum depth of 64 are supported.

See [`docs/SOURCE_INCLUDES_HU.md`](docs/SOURCE_INCLUDES_HU.md).

## Examples

All handwritten assembly examples live under [`examples/`](examples/), grouped by target architecture. The runtime crate intentionally does not host assembly examples.

Reusable ISA-specific source libraries are under `svm_asm/lib/<arch>/`; see `svm_asm/docs/CONSOLE_LIBRARY_HU.md` for the console helpers.

2-bpp graphics helpers and palette model: [`docs/GRAPHICS_LIBRARY_HU.md`](docs/GRAPHICS_LIBRARY_HU.md).

Per-ISA character-screen helper libraries are under `svm_asm/lib/<arch>/textscreen.asm`; see `svm_asm/docs/TEXT_SCREEN_LIBRARY_HU.md`.
