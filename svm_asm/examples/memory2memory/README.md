# memory2memory example

Assemble from the workspace root with:

`cargo run -p svm-asm -- memory2memory svm_asm/examples/memory2memory/hello.asm`

Then run the generated executable with `svm-rt`.
All standalone entry routines and callable helpers in these examples use `.proc` / `.endproc`, so the examples exercise assembler procedure-GC as well as the target ISA.

