# Fordítás és használat

A gyökérkönyvtár Cargo workspace.

```sh
cargo build --workspace --release
```

Fő binárisok:

- `target/release/svm-asm` — assembler, kilenc ISA
- `target/release/svm-rt` — közös runtime, CPU automatikus felismeréssel
- `target/release/svm-c` — közös SVM-C fordító (`-O0`, `-O1`, `-O2`, `-Os`)
- `target/release/svm-c-unopt-only` — optimalizáció nélküli referenciafordító

## Példa: C -> accumulator -> futtatás

```sh
target/release/svm-c -O2 --target accumulator svm_c/examples/fft_q15.sc
target/release/svm-rt svm_c/examples/fft_q15.sva
```

## Példa: assembly

```sh
target/release/svm-asm register svm_asm/examples/register/text_demo.asm
target/release/svm-rt svm_asm/examples/register/text_demo.svm
```

A runtime a fájl 4 byte-os magic mezőjéből dönti el, melyik CPU-magot kell használni.

## Példa: TTA16 assembly

```sh
target/release/svm-asm tta svm_asm/examples/tta/basic.asm basic.svt
target/release/svm-rt basic.svt
```
