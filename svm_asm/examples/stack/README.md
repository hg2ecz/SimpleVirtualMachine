# Examples

These examples target the current ISA and machine profile.

- `hello.fsasm` where present: minimal output/runtime smoke test.
- `memmove.fsasm`: overlap-safe forward/backward memory walking.
- Other examples demonstrate video or architecture-specific features.

For the cross-architecture integer benchmark, use `../../../svm_c/examples/fft_q15.sc` with the corresponding SVM-C target.
All standalone entry routines and callable helpers in these examples use `.proc` / `.endproc`, so the examples exercise assembler procedure-GC as well as the target ISA.

