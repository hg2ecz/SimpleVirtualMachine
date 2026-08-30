# Platform and ISA documentation

This directory contains the active, cross-component platform and ISA documentation. Component-specific material lives under `svm_asm/docs/`, `svm_rt/docs/`, and `svm_c/docs/`.

**Language-parity rule:** whenever an English and Hungarian document form an explicit pair, both must describe the same normative features, addresses, encodings, ABI rules and limitations. Explanatory wording and examples may differ, but neither language may be a reduced specification.

## Core platform references

- [`PLATFORM.md`](PLATFORM.md) - shared platform overview in English.
- [`PLATFORM_HU.md`](PLATFORM_HU.md) - shared platform overview in Hungarian.
- [`MMIO_REFERENCE_HU.md`](MMIO_REFERENCE_HU.md) - CPU MMIO register map.
- [`VIDEO_TEXT_REFERENCE_HU.md`](VIDEO_TEXT_REFERENCE_HU.md) - VRAM, 2 bpp graphics, palette and text rendering.
- [`BUILD_HU.md`](BUILD_HU.md) - workspace build notes.

## ISA references

- [`ISA_REFERENCE_EN.md`](ISA_REFERENCE_EN.md) / [`ISA_REFERENCE_HU.md`](ISA_REFERENCE_HU.md) - common ISA reference.
- [`ISA_CAPABILITY_MATRIX_HU.md`](ISA_CAPABILITY_MATRIX_HU.md) - comparison matrix for the nine architectures.
- [`ARCHITECTURE_DESIGN_RATIONALE_HU.md`](ARCHITECTURE_DESIGN_RATIONALE_HU.md) - architecture selection, hardware-assist, instruction-retention and Stack-cache rationale.
- [`IMPLEMENTATION_STATUS_HU.md`](IMPLEMENTATION_STATUS_HU.md) - current implementation status.

Architecture-specific normative specifications remain in this directory (`*_ISA_SPEC_HU.md`).

## Historical material

Development-time reviews and audits are kept under [`history/`](history/). They are not normative specifications.
