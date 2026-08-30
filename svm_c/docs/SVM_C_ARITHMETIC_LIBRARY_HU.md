# SVM-C arithmetic könyvtár

Az `svm_c/lib/arithmetic.sc` egy kis, preprocesszor nélküli ernyőkönyvtár.
A cél nem a teljes ANSI C `libm` másolása, hanem a 16 bites SVM platformon jó
ár/értékkel megvalósítható alapok biztosítása.

Használat:

```c
include "arithmetic.sc";
```

és például:

```sh
svm-c --target register -O2 -I svm_c/lib program.sc program.svm
```

## Modulok

- `arithmetic_int.sc`: `abs`, `min`, `max`, `clamp`, `isqrt`, `powu`, `gcd`, `lcm`
- `q15.sc`: `q15_abs`, `q15_neg`, `q15_mul`, `q15_div`
- `trig.sc`: `sin`, `cos`, `tan`
- `random.sc`: `srand`, `rand`, `rand_max`, `rand_range`
- `hrandom.sc`: közös MMIO RNG elérés: `hrand`, `hrand_seed`, `hrand_range`
- `signed_int.sc`: signed 8/16 bites segédek
- `wide_int.sc`: cím-alapú i32/u32 aritmetika és 32×32→64 teljes szorzat
- `f16.sc`, `f32.sc`, `float.sc`: software IEEE binary16/binary32 alapműveletek

Az egyes modulok külön is include-olhatók, ha a programnak nincs szüksége az egész
könyvtárra.

## Fixpontos trigonometria

A trigonometrikus `sin/cos/tan` rutinok továbbra is a kis költségű Q15 utat használják; nem ANSI `double` függvények. A külön `f16.sc` és `f32.sc` soft-float könyvtár IEEE binary16/binary32 bitmintákon ad alapműveleteket.

A szög 16 bites teljes-kör reprezentációt használ:

| érték | szög |
|---:|---:|
| `0x0000` | 0 fok |
| `0x4000` | 90 fok |
| `0x8000` | 180 fok |
| `0xC000` | 270 fok |

A `sin`, `cos` és `tan` visszatérési értéke signed Q15, amely `u16` bitmintában
van tárolva. `0x7fff` közel +1.0, `0x8000` -1.0.

A `sin`/`cos` kis költségű korrigált parabolikus közelítést használ. Nem igényel
lookup táblát vagy külön ROM-ot. A cél gyors, kis méretű általános függvény, nem
nagy pontosságú numerikus `libm`.

A `tan` a Q15 tartomány miatt `|tan(x)| >= 1` környezetében telítődik. A pólusok
közelében ezért nem matematikai lebegőpontos eredményt ad.

## Random

A `rand()` 0..32767 tartományt ad. A `srand(seed)` állítja a globális 16 bites LCG
állapotát. Ez gyors, determinisztikus PRNG; kriptográfiai célra nem alkalmas.

## ANSI-hasonlóság és eltérések

A `abs`, `rand`, `srand`, `sin`, `cos`, `tan` nevek szándékosan C-szerűek, de a
16 bites, lebegőpont nélküli nyelv miatt a típus- és tartomány-szemantika SVM-specifikus.
A `powu` és `isqrt` külön nevet kapott, hogy ne sugalljon lebegőpontos ANSI `pow`
/`sqrt` kompatibilitást.

### Hardverrel segített random (`hrandom.sc`)

A szoftveres, ANSI-szerű `rand()/srand()` mellett külön hardveres felület is van:

```c
u16 x;
hrand_seed(1234);   // opcionális, reprodukálható sorozathoz
x = hrand();        // 0..65535
x = hrand_range(10);
```

A hardveres RNG a közös MMIO perifériát olvassa; nem ISA-specifikus utasítás. **A jelenlegi VM periféria xorshift32 PRNG, tehát hardver/periféria-szintű, de nem valódi entrópiaforrás és nem kriptográfiai RNG.** Fizikai gépen valódi véletlenhez zaj-/jitteralapú entrópiaforrás szükséges; emulátorban host-OS entrópia köthető ugyanahhoz az MMIO interfészhez.
