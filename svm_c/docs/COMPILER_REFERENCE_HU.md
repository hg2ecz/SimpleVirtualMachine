# svm-c parancssori és fordítási referencia

## Parancssor

```text
svm-c --target <target> [-O0|-O1|-O2|-Os] [-I dir|-Idir] [--emit asm|bin] source.sc [output]
```

Targetek: `register`, `stack`, `accumulator`, `memreg`, `loadstore`, `regmem`, `memory2memory`, `belt`, `tta`.

`--emit bin` assembleren keresztül közvetlen futtatható SVM konténert készít. `--emit asm` az adott target olvasható assembly forrását írja ki.

## Optimalizáció

- `-O0`: AST-optimalizáció nélkül; minden parsed függvény megmarad.
- `-O1`: biztonságos konstans/algebrai egyszerűsítések, strength reduction, unreachable-function elimination.
- `-O2`: `-O1` + lokális constant/copy propagation és biztonságos dead-store elimination.
- `-Os`: méretorientált, biztonságos `-O2` transzformációk.

### Nem használt függvények

`-O1/-O2/-Os` esetén a fordító `main()`-ből induló call graph reachability pass-t futtat. Csak a tranzitívan elérhető függvények maradnak meg. Ez **a statikus memória-layout előtt** történik, ezért a dead függvények gépi kódot és statikus RAM-ot sem foglalnak. Egy `include` tehát forrásláthatóságot ad, de nem kényszeríti az egész könyvtár emittálását.

`-O0` és `svm-c-unopt-only` szándékosan megtart minden függvényt oktatási/referencia összehasonlításhoz.

## Include

```c
include "lib/graphics.sc";
```

Relatív keresés + `-I`, include-once, ciklusdetektálás és legfeljebb 64 szint. Részletesen: [`SOURCE_INCLUDES_HU.md`](SOURCE_INCLUDES_HU.md).

## ABI és memória

A C program alap load címe tipikusan `0x0100`. A compiler kis statikus/temporary objektumokat a low-memory régióban, nagyobb overflow statikus objektumokat a felső RAM-ban helyezhet el, miközben védi a programképet és a runtime stack-területet. A platform RAM fizikailag folyamatos; ezek compiler ABI-konvenciók.

A Load/Store és Register-Memory targeten `R6`, Memory-to-Memory targeten `A3` az egységes stack pointer: compiler temporary `PUSH/POP`, `CALL/RET` és IRQ ugyanazt a veremet használja.

## Típusok

Alaptípusok: `bool`, `u8/i8`, `u16/i16`, `u32/i32`, korlátozott `u64/i64` storage; `int == i16`, `long == i32`. A 32/64 bites és soft-float műveletek jelentős része address-based library ABI-t használ, hogy a CPU-k 16 bitesek maradhassanak.

Részletesen: [`NUMERIC_TYPES_HU.md`](NUMERIC_TYPES_HU.md), [`C_REFERENCE_HU.md`](C_REFERENCE_HU.md).
