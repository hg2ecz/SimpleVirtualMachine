# Runtime and virtual-machine documentation

This directory contains documentation specific to the `svm-rt` virtual machine and runtime implementation. Common platform and ISA specifications remain under the repository-level `docs/` directory.

## Runtime reference

- [`RUNTIME_USAGE_HU.md`](RUNTIME_USAGE_HU.md) - command-line execution, host window, keyboard, console and termination behavior.
- [`EXECUTABLE_FORMAT_HU.md`](EXECUTABLE_FORMAT_HU.md) - common 12-byte executable container, CPU magics, load/entry fields and payload.
- [`CYCLE_MODEL.md`](CYCLE_MODEL.md) - deterministic memory-access and multi-cycle operation accounting used by `svm-rt`.
- [`STRUCTURE_HU.md`](STRUCTURE_HU.md) - runtime structure, executable dispatch, memory implementation and CPU-core organization.

## Common hardware reference

- [`../../docs/PLATFORM_HU.md`](../../docs/PLATFORM_HU.md)
- [`../../docs/MMIO_REFERENCE_HU.md`](../../docs/MMIO_REFERENCE_HU.md)
- [`../../docs/VIDEO_TEXT_REFERENCE_HU.md`](../../docs/VIDEO_TEXT_REFERENCE_HU.md)

## Related documentation

- Common platform and ISA documentation: [`../../docs/README.md`](../../docs/README.md)
- Assembler documentation: [`../../svm_asm/docs/README.md`](../../svm_asm/docs/README.md)
- SVM-C compiler documentation: [`../../svm_c/docs/README.md`](../../svm_c/docs/README.md)
