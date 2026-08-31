# SVM architektúrák – implementációs állapot

A közös platformon jelenleg **kilenc CPU-architektúra** rendelkezik assembler- és runtime-úttal:

1. Register
2. Stack
3. Accumulator
4. MemReg
5. Load/Store
6. Register-Memory
7. Memory-to-Memory
8. Belt16
9. TTA16

Az újabb architektúrákhoz külön executable magic tartozik:

- Load/Store: `SVL\x01`
- Register-Memory: `SVR\x01`
- Memory-to-Memory: `SVC\x01`
- Belt16: `SVB\x01`
- TTA16: `SVT\x01`

Mindegyik elérhető a `svm-asm`, `svm-c` és `svm-rt` közös munkafolyamatában.

## C backend modell

A Load/Store és Register-Memory backend a meglévő register-expression loweringot használja, de a célassembler saját natív ISA-kódolást készít. A `PUSH/POP` compiler pseudo-opok nem új architekturális opcode-ok: az assemblerek ezeket a közös `R6` stack pointer módosítására és normál memóriautasításokra bontják. Ugyanezt az `R6`-ot használja a `CALL/RET` és az IRQ mentés is; nincs külön rejtett control-SP.

A Memory-to-Memory backend nem tartalmaz rejtett GPR-készletet. A register-expression lowering virtuális `R0..R7` temporaries-ei a `0x0000..0x000F` compiler-owned scratch memóriahelyekre kerülnek, ezért ennél a targetnél a C statikus objektumok kiosztása `0x0020`-tól indul. `A0..A3` kizárólag címregiszter; az implementált ABI-ban `A3` az egységes stack pointer a compiler temporaries, `CALL/RET` és IRQ számára.

## Include

Az include-rendszer mind a kilenc assembler/C target előtt lefut, ezért a saját függvénykönyvtárak architektúrától függetlenül használhatók.

## Assembler procedure-GC

Mind a kilenc assembler target közös `.proc/.endproc` eljárásmodellt használ. Az `.entry` és `.keep` gyökerekből, valamint az élő kódban szereplő szimbolikus hivatkozásokból felépített elérhetőségi gráf alapján a végső target assembler előtt kiesnek a nem használt eljárások. Az `svm_asm/lib/<arch>/` hívható rutinjai ezt a formát használják; a példaprogramok belépési és külön hívható rutinjai szintén `.proc` blokkok.

## Tudatosan későbbre hagyott optimalizációk

- automatikus branch relaxation a három új assemblerben;
- Register-Memory C backend közvetlen memory-source ALU loweringja;
- Load/Store C backend háromoperandusos kódgenerálásának további kihasználása;
- Memory-to-Memory expression tree-k közvetlen végcélba történő loweringja, amely csökkentheti a compiler scratch forgalmat.

Ezek teljesítmény/kódsűrűségi optimalizációk; az alapvető assembler/runtime/C funkcionalitáshoz nem szükségesek.

## C nyelv és könyvtárak

A közös `svm_c` frontend mind a kilenc targetet kezeli. Külön `svm-c-unopt-only` referenciaút mutatja az optimalizáló réteg nélküli fordítást. `-O1/-O2/-Os` alatt a `main()`-ből tranzitívan nem elérhető függvények még a statikus layout előtt kiesnek; `-O0` és `svm-c-unopt-only` minden beolvasott függvényt eljuttat a C-szintű kódgenerálásig. Bináris kimenetnél azonban minden fordítási módban lefut az assembler procedure-GC, ezért az el nem érhető generált `.proc` blokkok ekkor sem kerülnek a gépi kódba; `--emit asm` esetén viszont megmaradnak elemzéshez. A nyelv natív 8/16 bites skalárokat és cím-alapú 32/64 bites tárolási objektumokat támogat; `wide_int.sc`, `f16.sc` és `f32.sc` ad szoftveres többwordös/lebegőpontos aritmetikát.

## RNG

A közös MMIO RNG minden CPU-n azonos címen érhető el. A jelenlegi VM implementáció xorshift32 PRNG, ezért reprodukálható és olcsó, de nem valódi entrópiaforrás. Valódi hardveres véletlenhez fizikai zaj/jitter, emulátorban pedig host-OS entrópia szükséges; az MMIO interfész ettől nem változik.


## Integer aritmetikai segédletek

A többwordös egész és soft-float könyvtárakat kis költségű integer segédutasítások támogatják; hardveres floating point nincs. Lásd: `ARCHITECTURE_DESIGN_RATIONALE_HU.md`.

## Belt16

A Belt16 teljes assembler/runtime/C target útvonallal rendelkezik. Executable magic: `SVB\x01`.

- assembler: `svm_asm/src/belt/`
- runtime: `svm_rt/src/cpu/belt.rs`
- C backend: `svm_c/src/backend/belt.rs`
- assembler docs: `svm_asm/docs/belt/`
- ISA spec: `docs/BELT_ISA_SPEC_HU.md`

A C backend első változata `0x0000..0x000F` compiler-owned memóriacellákban tartja a közös virtuális temporaries-t, a felhasználói statikus kiosztás `0x0020`-tól indul. A valódi belt-élettartam optimalizálás külön későbbi backend-optimalizáció, nem hiányzó ISA-funkció.


## TTA16

- assembler: `svm_asm/src/tta/`
- runtime: `svm_rt/src/cpu/tta.rs`
- C backend: `svm_c/src/backend/tta.rs`
- assembler docs: `svm_asm/docs/tta/`
- examples: `svm_asm/examples/tta/`
- executable magic: `SVT\x01`

A TTA16 core utasítása adattranszport. Az ALU, memória, VRAM és vezérlés portokra történő írásból indul; a C backend a közös frontend után explicit transportokra bontja a műveleteket.


## Numerikus regressziós referencia

A v2.3.17 referenciafuttatásban a `svm_c/examples/smoke/smoke_all.sc` és `smoke_all_compact.sc` mind a kilenc targeten `OK`. Ez a numerikus könyvtárak és a kilenc backend közös regressziós alapállapota; a részletes `smoke_*.sc` tesztek hibakeresésre és rétegenkénti ellenőrzésre használhatók.
