# svm-rt

One runtime, nine CPU cores, one shared machine platform.

Run any executable directly; the 4-byte executable magic selects the CPU automatically:

```sh
cargo run -p svm-rt --release -- program.sva
cargo run -p svm-rt --release -- program.svs
```

Shared: 64 KiB system address space, MMIO, separate 16 KiB VRAM, 320x200x2bpp video, palette, keyboard, VT100 console, timer/IRQ and cycle counters. There is no guest-visible System ROM.

RNG note: the current reference MMIO RNG is a deterministic xorshift32 hardware-PRNG model. It is not an entropy source; a physical target may attach noise/jitter entropy and an emulator may attach host-OS entropy to the same MMIO interface.

## Assembly examples

The runtime crate no longer owns assembly examples. All hand-written assembly examples are under `../svm_asm/examples/<cpu>/`, grouped by ISA.

## Stack CPU cache

The Stack core uses an internal two-cell `TOS`/`NOS` lazy stack cache. This does not change the ISA; it only removes real data-stack RAM accesses when the top two logical stack items are already cached. See `../docs/ARCHITECTURE_DESIGN_RATIONALE_HU.md`.

## Documentation

Runtime and virtual-machine documentation is indexed in `docs/README.md`. The detailed cycle accounting model is in `docs/CYCLE_MODEL.md`.

## Reference map

- Runtime usage: [`docs/RUNTIME_USAGE_HU.md`](docs/RUNTIME_USAGE_HU.md)
- Executable format: [`docs/EXECUTABLE_FORMAT_HU.md`](docs/EXECUTABLE_FORMAT_HU.md)
- Cycle model: [`docs/CYCLE_MODEL.md`](docs/CYCLE_MODEL.md)
- MMIO: [`../docs/MMIO_REFERENCE_HU.md`](../docs/MMIO_REFERENCE_HU.md)
- Video/text: [`../docs/VIDEO_TEXT_REFERENCE_HU.md`](../docs/VIDEO_TEXT_REFERENCE_HU.md)
