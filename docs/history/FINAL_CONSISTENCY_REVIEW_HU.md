# Végső konzisztencia-felülvizsgálat

A jelenlegi referenciaállapot kilenc 16 bites CPU-architektúrát tartalmaz: Register, Stack, Accumulator, MemReg, Load/Store, Register-Memory, Memory-to-Memory, Belt16 és TTA16.

A korábbi v2.2.8 workspace-teszt volt az első teljesen zöld alapállapot. Azóta több implementációs és regressziós kör történt; a v2.3.17 referenciafuttatásban a `smoke_all.sc` és a `smoke_all_compact.sc` mind a kilenc CPU-targeten `OK`. A jelen dokumentációs audit a v2.3.17 implementált állapotát tekinti normatívnak.

## Ellenőrzött területek

- assembler target-lista és a kilenc architektúra kézikönyv/reference párosa;
- runtime CPU-magok, közös platform, külön VRAM, MMIO és System ROM nélküli modell;
- SVM-C common frontend, kilenc backend, optimalizált és `unopt-only` belépési pont;
- Register compact `AND`, MemReg hot `AND`, Load/Store natív `SUBI16` és carry/no-borrow szemantika;
- Stack assembly-orientált utasítások státusza;
- `bool/i8/u8/i16/u16/i32/u32/i64/u64`, soft `f16/f32`, wide-int és RNG dokumentáció;
- Belt16 és TTA16 aktív referencia- és példadokumentáció;
- assembly példák `svm_asm/examples/` alatti közös elhelyezése és C példaútvonalai.

## Normatív belépési pontok

- `docs/ISA_REFERENCE_HU.md` / `docs/ISA_REFERENCE_EN.md`
- `docs/PLATFORM_HU.md` / `docs/PLATFORM.md`
- `docs/IMPLEMENTATION_STATUS_HU.md`
- `svm_asm/docs/<cpu>/`
- `svm_c/docs/C_REFERENCE_HU.md` / `C_REFERENCE_EN.md`
- `svm_c/docs/NUMERIC_TYPES_HU.md`

A korábbi v0.x/v1.x tervezési dokumentumok megmaradhatnak történeti háttérként; az azokban szereplő „tervezet”, korábbi architektúraszám vagy elvetett javaslat nem írja felül a fenti aktuális referenciákat.

## 2.3.5 - program-szintu kodmeret-optimalizalas

Az optimalizalo `svm-c` `-O1/-O2/-Os` szinteken a `main()`-bol kozvetlen vagy tranzitivan nem elerheto fuggvenyeket mar nem emittalja. Ez kulonosen az include-olt aritmetikai/soft-float konyvtaraknal fontos: a forrasban szereplo, de nem hasznalt rutinok nem novelik a gepi kodot. Az `-O0` es az `svm-c-unopt-only` ezt szandekosan nem vegzi el, hogy az optimalizalas nelkuli fordito egyszerusege es kodmerete osszehasonlithato maradjon.

A RegMem, Memory-to-Memory, Belt16 es TTA16 assembler a folytonos programkep MMIO-tartomanyba (`0xFF00..`) logasat forditasi hibakent kezeli, nem engedi csendben periferiairassal felulirni a programkepet.

## v2.3.8 soft-float korrekcio

Az f32 smoke teszt minden architekturan ugyanazon `f32_add` hibaval allt meg. A kozos
soft-float utvonal felulvizsgalata ket javitast eredmenyezett:

- `u32_shr1`: a felso wordbol csak a bit0 kerulhet az also word bit15 helyere;
- `f32_add`: a binary32 24 bites mantissza osszeadasa/kivonasa kozvetlen ketwordos
  (`hi:lo`) algoritmust hasznal, felesleges u32 temporary/helper lanc nelkul.

A CPU-k ISA-ja es runtime-ja nem valtozott.

## v2.3.9 numerikus smoke regressziós készlet

A numerikus könyvtárakhoz célzott, kis futásidejű smoke programok kerültek a
`svm_c/examples/smoke/` könyvtárba. Ezek az FFT példáktól függetlenül ellenőrzik
a natív 8/16 bites egész műveleteket, a `u32/i32` wide-int réteget, a kizárólag
32x32->64 eredményre használt `u64/i64` tárolást, a Q15/trig réteget, valamint az
`f16` és `f32` soft-float primitíveket és konverziókat. A `smoke_all.sc` egyetlen
gyors regressziós kapuként a legfontosabb ellenőrzéseket együtt futtatja.

A tesztek bitpontos referenciaértékeket használnak, ahol ez értelmes. A szoftveres
PRNG determinisztikus referencia-sorozattal ellenőrzött; a hardveres/MMIO
entrópiaforrás nincs a determinisztikus smoke kapuba kötve.

## v2.3.11 - O2 self-update DSE regresszió javítása

Az O2 lokális dead-store elimináció korábban tévesen eltávolíthatta az előző
értékadást ilyen mintánál:

```c
hi = ah + bh;
hi = hi + carry;
```

A második értékadás olvassa `hi` korábbi értékét, ezért az első store nem dead.
Az optimalizáló most csak akkor törli a közvetlen előző értékadást vagy
inicializálást, ha az új jobb oldal nem olvassa ugyanazt a változót. Célzott
regressziós tesztek védik az assignment, initializer és valódi overwrite
eseteket.


## v2.3.12 - Stack C boolean normalization és smoke pontosítás

- A Stack ISA Forth-kompatibilis összehasonlításai igaz esetben `0xFFFF`-et adnak, miközben az SVM-C relációs operátorainak numerikus `1`-et kell eredményezniük. A Stack C backend ezért az `=`, `<>`, `U<`, `U>` és ezekből képzett `<=`, `>=` eredményét `1 AND` művelettel 0/1-re normalizálja. Ez javítja többek között a wide-int carry számítását (`carry = lo < al`).
- A Q15 trigonometrikus rutin közelítő algoritmus, ezért a smoke teszt a nevezetes pontokon kis, legfelj 4 LSB eltérést enged. A korábbi bitpontos `0x7FFF` elvárás tévesen bukott a dokumentált közelítő `sin/cos` megvalósításon.
- Optimalizált fordításnál az elérhetetlen függvények eltávolítása már a statikus címkiosztás előtt is lefut. Így az umbrella include-ok dead függvényeinek lokálisai nem foglalnak feleslegesen zero-page/high-static címtartományt; a normál optimizer a lowering után továbbra is lefut.


## v2.3.17 - egységes stack pointer és 9/9 numerikus smoke

A Load/Store és Register-Memory CPU-n az `R6`, a Memory-to-Memory CPU-n az `A3` az egyetlen stack pointer a compiler temporary `PUSH/POP`, `CALL/RET` és interrupt mentés számára. A korábbi külön rejtett control-SP dokumentáció elavult.

A `smoke_all.sc` és `smoke_all_compact.sc` referenciafuttatása mind a kilenc targeten sikeres. A felső MMIO-lap (`0xFF00..0xFFFF`) miatt a korábbi `0x7000` programkép-határ megszűnt; a CPU RAM `0x0000..0xFEFF` összefüggő.

Optimalizált (`-O1/-O2/-Os`) fordításnál a `main()`-ből tranzitívan el nem érhető függvények még a statikus layout előtt kiesnek. `-O0` és `svm-c-unopt-only` továbbra is minden beolvasott függvényt megtart.
