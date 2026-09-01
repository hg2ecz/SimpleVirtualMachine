# SVM C-Lite learning path

C-Lite is meant to be understood at three levels: structured C-Lite source, target-neutral CLIR, and finally target-specific assembly. Normal programming requires only the first level; the other two are for learning and debugging.

Start with `svm-clite --check source.cl`, then inspect `--emit ir`, and only later compare `--emit asm` output for two architectures. The recommended progression is arithmetic, variables, branches, loops, arrays/pointers, calls, MMIO, then target assembly.

CLIR uses virtual temporaries such as `%0`, explicit `load/store`, `addr/index/loadmem/storemem`, labels and jumps, and `call/ret`. These are intentionally architecture-neutral: no physical registers, stack opcodes, belt positions or TTA buses appear in CLIR.

## Separating CLIR from target code

`--emit ir` shows language lowering, while `--emit asm` shows the selected architecture backend result. C-Lite deliberately has no optimizer. The selected target assembly is a direct, mechanical lowering of the same CLIR. See `CODEGEN_EN.md`.
