# SVM-C optimalizálási referencia

## Cél

Az SVM-C kis, auditálható optimalizáló passzokat használ. A cél jó kódméret és végrehajtási költség SSA/CFG alapú, nagy komplexitású optimizer nélkül.

A kötelező lowering nem optimalizáció: minden olyan átalakítás ide tartozik, amely szükséges ahhoz, hogy a közös AST a kilenc backend egyikére természetesen leképezhető legyen.

## Fordítási szintek

- `-O0` – nincs opcionális AST-optimalizáció; alkalmas közvetlen forrás→assembly vizsgálatra.
- `-O1` – biztonságos lokális egyszerűsítések és konstanshajtás.
- `-O2` – `-O1` plusz lokális konstans/copy propagation, egyszerű dead-store elimináció és target-aware helyi javítások.
- `-Os` – méretorientált optimalizált pipeline, a legkisebb elérhető kódolások előnyben részesítésével.
- `svm-c-unopt-only` – külön referencia belépési pont, amely nem deklarál és nem hív opcionális optimizer modult, és nem fogad `-O` kapcsolót.

## Nem használt függvények eltávolítása

`-O1`, `-O2` és `-Os` esetén a fordító a `main()`-ből induló közvetlen hívási gráfot tranzitívan bejárja. Csak az elérhető függvények maradnak meg.

A passz **a statikus memória-kiosztás előtt** fut, ezért a dead függvények:

- nem generálnak gépi kódot;
- lokális/statikus compiler-objektumaik nem foglalnak RAM-ot;
- egy nagy könyvtár `include`-ja önmagában nem növeli a programképet az összes könyvtári rutinnal.

`-O0` és `svm-c-unopt-only` szándékosan megtart minden beolvasott függvényt az oktatási és kódgenerálási összehasonlíthatóság érdekében.

A nyelv jelenleg nem támogat függvénypointert, ezért a direct-call reachability teljes a támogatott nyelvi modellen belül.

## Közös AST-optimalizálások

Tipikus példák:

- konstanshajtás (`2 + 3 -> 5`);
- semleges elemek eltávolítása (`x + 0`, `x * 1`);
- natív `INC/DEC` alakok felismerése;
- biztonságos power-of-two strength reduction;
- konstans összehasonlítás és konstans control-flow egyszerűsítés;
- `if (0)` / `while (0)` eltávolítás;
- `-O2` lokális konstans- és copy propagation;
- közvetlenül felülírt, mellékhatásmentes store eltávolítása.

A DSE csak akkor törölhet egy korábbi értékadást, ha a következő RHS nem olvassa ugyanazt a változót. Ez megőrzi az olyan helyes mintákat, mint:

```c
hi = ah + bh;
hi = hi + carry;
```

Függvényhívás és control-flow join konzervatívan érvényteleníti a lokális értéktudást; a compiler nem végez teljes alias- vagy globális data-flow analízist.

## Loop-condition biztonság

`-O2` és `-Os` nem fagyasztja be a ciklus előtt ismert változóértéket a `while` vagy `for` ismételten kiértékelt feltételébe. Az indukciós változók futásidejű betöltése megmarad.

## Backend-szintű instrukcióválasztás

Nem minden rövidülés optimizer-passz. Ha az ISA natív, közvetlen formát ad egy AST műveletre, annak használata kötelező lowering/instruction selection, ezért `-O0` mellett is történhet.

Példák:

- Register: `ADDI`, natív `SUBI`, `CMPI`, compact GPR formák, `INC/DEC/SHL1/SHR1`;
- MemReg: `ADDI/SUBI/ANDI/ORI/XORI/CMPI`, valamint `0x000E..0x000F` hot expression scratch;
- Stack: rövid literalok, egybájtos ALU/stack primitívek, TOS/NOS cache-t jól kihasználó lowering;
- lineáris memóriajárásnál az adott ISA természetes post-increment/pre-decrement alakjai.

A konstans sztringes `puts()` címét a Register és MemReg backend csak egyszer tölti be a kiírási ciklus előtt.

## MemReg hot scratch

MemReg célon a compiler a `0x000E..0x000F` 16 bites párt expression scratch célra foglalja. Ez a hot direct-file tartomány része, ezért a gyakori `MOV16`, `ADD` és `AND` műveletek rövidebb kódolást kaphatnak.

A `0x0000..0x000D` továbbra is felhasználói statikus terület; a scratch más target layoutját nem módosítja.

## `svm-c-unopt-only`

A referenciafordító ugyanazt a frontendet, szemantikát, ABI-t és backendeket használja, mint a normál `svm-c`, de kihagyja az opcionális optimizer-réteget.

Használat:

```bash
svm-c-unopt-only --target register program.sc program.svm
svm-c-unopt-only --target stack -I lib --emit asm program.sc program.asm
```

Az `-O0/-O1/-O2/-Os` kapcsolók ezen a binárison szándékosan hibát jelentenek.

## Forrásszerkezeti határ

A közös részek:

```text
common/      lexer/parser/AST/semantic/layout
backend/     a kilenc ISA kódgenerálása
optimized/   opcionális optimizer + optimalizált pipeline
unopt/       optimizer nélküli pipeline
```

A nyelvi szabályok és a statikus layout közösek; az instrukcióválasztás és ISA-specifikus ABI-részletek a backendek feladatai.

## Tudatosan kihagyott optimalizálások

Jelenleg nincs:

- SSA;
- globális liveness és globális register allocation;
- agresszív inlining;
- loop unrolling;
- komplex alias analysis;
- nagy interprocedurális optimizer.

Ilyen funkció csak akkor indokolt, ha valós programokon a nyereség egyértelműen meghaladja a compiler komplexitásnövekedését.

## Kapcsolódó dokumentumok

- [Compiler referencia](COMPILER_REFERENCE_HU.md)
- [C nyelvi referencia](C_REFERENCE_HU.md)
- [Numerikus típusok](NUMERIC_TYPES_HU.md)
- [Smoke tesztek](SMOKE_TESTS_HU.md)
