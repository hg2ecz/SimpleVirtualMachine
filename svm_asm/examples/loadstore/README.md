# loadstore example

Assemble from the workspace root with:

`cargo run -p svm-asm -- loadstore svm_asm/examples/loadstore/hello.asm`

Then run the generated executable with `svm-rt`.
All standalone entry routines and callable helpers in these examples use `.proc` / `.endproc`, so the examples exercise assembler procedure-GC as well as the target ISA.

