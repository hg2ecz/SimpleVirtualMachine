# Dokumentációs felülvizsgálat – aktuális állapot

Ez a dokumentum a platform aktuális, implementált állapotának rövid indexe. A történeti tervezési dokumentumok megmaradnak, de ahol korábbi v0.x döntést vagy implementáció előtti állapotot írnak le, azt történeti kontextusként kell olvasni.

## Aktuális architektúrák

A platform kilenc 16 bites CPU-modellt tartalmaz:

1. Register
2. Stack
3. Accumulator
4. MemReg
5. Load/Store
6. Register-Memory
7. Memory-to-Memory
8. Belt16
9. TTA16

Mind a kilenc rendelkezik assembler-, runtime- és SVM-C target útvonallal. A közös perifériamodell és VRAM nem ISA-specifikus.

## Aktuális C fordító

Egyetlen `svm_c` crate van:

- `svm-c`: `-O0/-O1/-O2/-Os`;
- `svm-c-unopt-only`: külön, optimizer nélküli oktatási/reference pipeline.

A common frontend/szemantika/layout közös; a kilenc ISA külön backend fájlt kap. Az include preprocessing nélküli forrás-include: `include "file.sc";` és `-I` keresési út támogatott.

## Numerikus modell

Natív skalárok: `bool`, `i8`, `u8`, `i16`, `u16`; `int` = `i16`.

Wide address-only objektumok: `i32`, `u32`, `i64`, `u64`; `long` = `i32`. A 32 bites aritmetika könyvtári, a 64 bites típus elsődlegesen a 32×32 teljes szorzat eredményének tárolója. Nincs általános 64 bites publikus aritmetikai API.

Soft-float:

- `f16.sc`: IEEE-754 binary16 bitminta `u16`-ban;
- `f32.sc`: IEEE-754 binary32 bitminta `u32` wide objektumban.

A CPU ISA-k továbbra sem kapnak lebegőpontos utasításokat.

## RNG

Két külön fogalom van:

- `random.sc`: tisztán szoftveres, determinisztikus PRNG;
- `hrandom.sc` + MMIO: periféria-szintű RNG interfész.

A jelenlegi VM referenciaimplementáció az MMIO perifériában is determinisztikus `xorshift32` PRNG-t használ. Ez **nem valódi entrópiaforrás**. Fizikai gépen zaj-/jitteralapú entrópia, emulátorban host-OS entrópia köthető ugyanahhoz az MMIO interfészhez, ha nemdeterminisztikus véletlen szükséges.

## Aktuális elsődleges dokumentumok

- `PLATFORM_HU.md` / `PLATFORM.md` – memória, MMIO, VRAM, RNG;
- `FINAL_CONSISTENCY_REVIEW_HU.md` – a zöld workspace-teszt utáni dokumentációs lezárás;
- `IMPLEMENTATION_STATUS_HU.md` – kilenc ISA implementációs állapota;
- `SVM_C_UNIFICATION_HU.md` – C fordító struktúrája;
- `../svm_asm/docs/SOURCE_INCLUDES_HU.md` – assembler include mechanizmus;
- `../svm_c/docs/SOURCE_INCLUDES_HU.md` – SVM-C include mechanizmus;
- `svm_c/docs/C_REFERENCE_HU.md` / `C_REFERENCE_EN.md` – nyelvi referencia;
- `svm_c/docs/NUMERIC_TYPES_HU.md` – integer/soft-float modell;
- `SVM_C_ARITHMETIC_LIBRARY_HU.md` – arithmetic könyvtár;
- `STACK_CACHE_DESIGN_HU.md` – a Stack CPU TOS+NOS lazy stack-cache mikroarchitektúrája;
- az egyes ISA-specifikációk – konkrét opcode/ABI leírás.

## Memória- és stackmodell

A CPU-címtérben `0x0000..0xFEFF` fizikailag összefüggő RAM, az MMIO kizárólag a felső `0xFF00..0xFFFF` lapon van. A runtime konvenció a felső 1 KiB RAM-ot (`0xFB00..0xFEFF`) stackre tartja fenn. Stack/Belt/TTA ezt data és return/control részre osztja; a többi CPU egyetlen lefelé növő stackként használja.

A v2.3.17-ben a Load/Store és Register-Memory `R6`, illetve a Memory-to-Memory `A3` lett az egységes stack pointer a compiler temporaries, `CALL/RET` és IRQ számára. Nincs külön rejtett második SP, amely ugyanabba a RAM-területbe egymástól függetlenül írhatna.

## Optimalizálási elv

A projekt nem épít külön benchmark-/telemetria-alrendszert. Elsődlegesek a statikusan igazolható, kis komplexitású optimalizálások: redundáns opcode helyett alias, meglévő immediate/compact/memory-source formák jobb compiler-kihasználása, peephole és assembler-oldali egyszerűsítés.

`-O1/-O2/-Os` alatt a `main()`-ből tranzitívan nem elérhető függvények még a statikus layout előtt kiesnek. Így az include-olt, de nem használt könyvtári rutinok sem kódot, sem statikus RAM-ot nem foglalnak. `-O0` és `svm-c-unopt-only` ezt szándékosan nem végzi el.

## Aktuális ISA referencia

A kilenc CPU közös belépési pontja: `docs/ISA_REFERENCE_HU.md`. Az assemblerhez mind a kilenc architektúrán külön `ASSEMBLY_PROGRAMMING_MANUAL_HU.md` és `INSTRUCTION_REFERENCE_HU.md` található az `svm_asm/docs/<cpu>/` könyvtárban. A Register compact logikai kódolás `AND`, a MemReg hot logikai kódolás `AND`, a Load/Store `SUBI` pedig külön hosszú-immediate dekódot használ a carry/no-borrow helyesség miatt.


## v2.3.18 dokumentációs audit

A v2.3.17 numerikus regressziós állapotban a `smoke_all.sc` és a `smoke_all_compact.sc` mind a kilenc targeten `OK`. A dokumentációs audit ennek megfelelően egységesítette:

- a `0x0000..0xFEFF` összefüggő RAM + `0xFF00..0xFFFF` MMIO térképet;
- a `0xFB00..0xFEFF` runtime stack-konvenciót;
- a Stack TOS+NOS lazy cache leírását;
- a Load/Store, Register-Memory és Memory-to-Memory egységes stack-pointer modelljét;
- az `-O1/-O2/-Os` unreachable-function eliminációt és a layout előtti végrehajtását;
- a numerikus smoke rendszer 9-targetes referenciaállapotát.
