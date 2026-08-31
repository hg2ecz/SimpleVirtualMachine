# Belt16 assembly examples

The examples are intentionally small and each demonstrates one Belt16 idea.

- `basic.asm` - minimal `LDI/ADD` sequence.
- `arithmetic.asm` - belt ageing and chained arithmetic.
- `memory.asm` - indirect RAM load/store using a belt value as address.
- `loop.asm` - flags and conditional branch without a general register file.
- `call.asm` - `CALL/RET` while keeping the belt as the value interface.
- `carry32.asm` - 32-bit addition from 16-bit `ADD/ADC` plus the data stack.
- `video.asm` - VRAM and common video MMIO access.

Typical assembly command:

```sh
svm-asm belt svm_asm/examples/belt/arithmetic.asm arithmetic.svb
```

The C examples under `svm_c/examples/` are target-independent and can also be
compiled for Belt16, for example:

```sh
svm-c --target belt -O2 svm_c/examples/hello.sc hello.svb
svm-c-unopt-only --target belt svm_c/examples/hello.sc hello-unopt.svb
```
All standalone entry routines and callable helpers in these examples use `.proc` / `.endproc`, so the examples exercise assembler procedure-GC as well as the target ISA.

