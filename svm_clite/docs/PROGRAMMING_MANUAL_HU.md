# SVM C-Lite programozói kézikönyv

**Verzió:** 1.0  
**Státusz:** Teljes magyar társ-dokumentáció; a normatív elsődleges változat a `PROGRAMMING_MANUAL_EN.md`  
**Nyelvi cél:** strukturált, architektúrafüggetlen assembly  
**Célarchitektúrák:** Register, Stack, Accumulator, MemReg, Load/Store, Register-Memory, Memory-to-Memory, Belt, TTA

## 1. Mi a C-Lite?

Az SVM C-Lite egy szándékosan kicsi, C-szerű programozási nyelv a SimpleVirtualMachine architektúráihoz. Nem teljes C és nem Rust-részhalmaz. A célja az, hogy assembly-szintű programokat lehessen írni **architektúraspecifikus assembly szintaxis megtanulása nélkül**, néhány jól ismert strukturált nyelvi elemmel.

A nyelv legfontosabb tervezési elve:

> A C-Lite strukturált, architektúrafüggetlen assembly. A nyelv és a fordító egyszerűsége fontosabb a C-kompatibilitásnál vagy az optimalizált kódnál.

A forrás például ugyanaz marad mind a kilenc célarchitektúrán:

```c
fn add(u16 a, u16 b) -> u16 {
    return a + b;
}

fn main() -> u16 {
    return add(10, 20);
}
```

A fordítási modell:

```text
C-Lite forrás
    ↓
lexer
    ↓
parser
    ↓
egyszerű szemantikai ellenőrzés
    ↓
CLIR 0.1
    ↓
mechanikus target ASM
    ↓
külső svm-asm
    ↓
SVM program
```

A C-Lite fordító nem tartalmaz optimalizálót, SSA-t, regiszterallokátort, linkert vagy beépített assemblert.

---

## 2. Első program

```c
fn main() -> u16 {
    u16 a = 10;
    u16 b = 20;
    return a + b;
}
```

Ellenőrzés fordítás nélkül:

```sh
svm-clite --check hello.cl
```

Assembly generálás Register targetre:

```sh
svm-clite --target register hello.cl
```

Az eredmény alapértelmezésben `hello.asm`.

Gépi program készítése külön assemblerrel:

```sh
svm-asm register hello.asm hello.svm
```

Kényelmi módban a C-Lite a külső assemblert is meghívhatja:

```sh
svm-clite --target register --assemble hello.cl hello.svm
```

Az assembler más útvonalon is lehet:

```sh
svm-clite --assembler /opt/svm/bin/svm-asm --target stack --assemble hello.cl
```

---

## 3. Támogatott targetek

A `--target` kapcsoló értékei:

| Target | Rövid alias |
|---|---|
| `register` | `reg` |
| `stack` | – |
| `accumulator` | `acc` |
| `memreg` | – |
| `loadstore` | – |
| `regmem` | – |
| `memory2memory` | `m2m` |
| `belt` | – |
| `tta` | – |

Ugyanazt a `.cl` forrást célszerű minden targetre változtatás nélkül fordítani.

---

## 4. Forrásfájl és kommentek

A javasolt kiterjesztés:

```text
.cl
```

Egysoros komment:

```c
// ez egy komment
u16 x = 10;
```

Többsoros komment:

```c
/*
   több soros
   komment
*/
```

A kommentek nem kerülnek be az AST-ba vagy a generált assemblybe.

---

## 5. Típusok

A nyelv teljes alaptípuskészlete:

```text
bool
i8
u8
i16
u16
void
```

### 5.1 Egész típusok

- `u8`: 8 bites előjel nélküli egész
- `i8`: 8 bites előjeles egész
- `u16`: 16 bites előjel nélküli egész
- `i16`: 16 bites előjeles egész

A C-Lite nem tartalmaz `i32`, `u32`, lebegőpontos vagy automatikus széles típusrendszert. Ha ilyen algoritmus szükséges, könyvtárban több szóból vagy assembly rutinból megvalósítható.

### 5.2 `bool`

```c
bool ready = true;
bool done = false;
```

A `bool` logikai jelentésű. Memóriában egy byte-ot foglal és a tárolt értéke 0 vagy 1.

```c
bool flags[16];
```

Ez 16 byte, nem 16 bit. Nincs bitcsomagolás.

Az összehasonlítások `bool` értéket adnak:

```c
bool smaller = a < b;
```

### 5.3 `void`

A `void` csak függvény visszatérési típusaként használható. Globális, lokális és paraméter nem lehet `void`.

Ha egy függvényfejben nincs `-> típus`, a függvény `void`:

```c
fn clear(u8* data, u16 count) {
    // void függvény
}
```

---

## 6. Számliterálok

Három forma támogatott:

```c
u16 a = 1234;       // decimális
u16 b = 0x04d2;     // hexadecimális
u16 c = 0b10101010; // bináris
```

A lexer a számliterált `u16` értékként kezeli, ezért a literálnak 0..65535 tartományba kell férnie.

Egész literál közvetlenül inicializálhat vagy kaphat `i8`, `u8`, `i16` vagy `u16` változót. A fordító jelenleg nem végez külön literál-tartományellenőrzést a kisebb cél-típusra; alacsony szintű nyelvként a programozó felelőssége, hogy megfelelő értéket használjon.

Nincs konstanshajtás:

```c
u16 x = 2 + 3 * 4;
```

több külön CLIR művelet marad.

---

## 7. Változók

### 7.1 Lokális változó

```c
u16 counter;
u16 start = 10;
u8 ch = 65;
bool active = true;
```

### 7.2 Globális változó

```c
u16 ticks;
u8 mode = 1;
bool enabled = false;
```

Globális skalár inicializáló csak közvetlen egész- vagy bool-literál lehet. Például ez helyes:

```c
u16 mode = 3;
```

Ez nem támogatott globális inicializáló:

```c
u16 mode = 1 + 2;
```

### 7.3 Egyszerű névtér

A C-Lite tudatosan nem enged névárnyékolást.

Ez hibás:

```c
u16 count;

fn f(u16 count) -> u16 {
    return count;
}
```

Egy függvényen belül minden paraméter és lokális név egyedi. Külön blokkokban sem használható újra ugyanaz a lokális név. Ennek oka, hogy a közvetlen backend minden lokálisnak egyszerű statikus memóriahelyet ad.

---

## 8. Fix tömbök

A tömbszintaxis C-szerű:

```c
u16 values[4];
u8 bytes[128];
i16 samples[32];
bool flags[8];
```

A tömb hossza fordítási idejű pozitív számliterál.

Nem támogatott:

- nulla hosszúságú tömb;
- beágyazott tömb;
- pointerelemű tömb;
- tömb inicializáló lista;
- tömbparaméter.

Függvényhez pointert kell átadni:

```c
fn sum(u16* data, u16 count) -> u16 {
    return data[0];
}
```

### 8.1 Indexelés

```c
u16 values[4];
values[0] = 10;
values[1] = values[0] + 1;
```

Konstans index esetén van határellenőrzés:

```c
values[4] = 1; // hiba, ha values[4] négy elemű
```

Dinamikus index esetén futásidejű határellenőrzés nincs:

```c
values[i] = 1;
```

A C-Lite alacsony szintű nyelv; a programozó felel az érvényes címzésért.

---

## 9. Pointerek

A nyelv egyetlen pointer-szintet támogat:

```c
u16* p;
u8* bytes;
bool* flags;
```

Nincs:

```text
void*
u16**
function pointer
```

### 9.1 Címképzés

```c
u16 x = 10;
u16* p = &x;
```

Tömbelem címe:

```c
u16 values[4];
u16* p = &values[0];
```

### 9.2 Dereferencia

```c
u16 x = 10;
u16* p = &x;
*p = 20;
return *p;
```

### 9.3 Pointer indexelés

```c
p[i]
```

A címnövekményt a fordító az elemtípus alapján számolja:

- `u8*`, `i8*`, `bool*`: 1 byte/elem
- `u16*`, `i16*`: 2 byte/elem

Nincs általános C pointeraritmetika, például `p + i` használatára nem érdemes építeni. Az indexelés a támogatott, egyszerű forma.

---

## 10. Függvények

### 10.1 Alapforma

```c
fn add(u16 a, u16 b) -> u16 {
    return a + b;
}
```

Void függvény:

```c
fn set_zero(u16* p) {
    *p = 0;
    return;
}
```

A `return;` void függvényben használható.

### 10.2 Paraméterek

```c
fn mix(u8 a, i8 b, u16 c, i16 d, bool enabled) -> u16 {
    if (enabled) {
        return c;
    }
    return 0;
}
```

Tömbparaméter nincs. Pointerparamétert kell használni.

### 10.3 Rekurzió

Közvetlen és kölcsönös rekurzió tiltott:

```c
fn f(u16 x) -> u16 {
    return f(x); // hiba
}
```

és:

```text
a -> b -> a
```

is hiba.

Ez lehetővé teszi az egyszerű, statikus paraméter- és lokális memóriahelyeket általános stack frame nélkül.

### 10.4 `main`

Minden programnak tartalmaznia kell `main` függvényt.

A tipikus forma:

```c
fn main() -> u16 {
    return 0;
}
```

---

## 11. Kifejezések és operátorok

### 11.1 Aritmetika

```text
+  -  *  /  %
```

Példa:

```c
u16 x = a + b * 2;
```

### 11.2 Bitműveletek

```text
&  |  ^  ~  <<  >>
```

Példa:

```c
u16 mask = 0x8000;
u16 bit = value & mask;
value = value << 1;
```

### 11.3 Összehasonlítás

```text
==  !=  <  <=  >  >=
```

Az eredmény `bool`.

```c
bool equal = a == b;
bool less = a < b;
```

### 11.4 Nincs logikai `&&`, `||`, `!`

A C-Lite szándékosan nem tartalmaz külön rövidzáras logikai operátorokat.

Használható például:

```c
if (a != 0) {
    if (b != 0) {
        // a és b is nem nulla
    }
}
```

A `~` bitenkénti NOT, nem logikai NOT.

### 11.5 Operátor-precedencia

Nagyobbtól kisebb felé:

1. unáris `-`, `~`, `&`, `*`
2. `* / %`
3. `+ -`
4. `<< >>`
5. `< <= > >=`
6. `== !=`
7. bitenkénti `&`
8. bitenkénti `^`
9. bitenkénti `|`

Zárójelezés javasolt, ha a kifejezés olvashatósága kérdéses.

---

## 12. Típuskezelés

A C-Lite nem akar teljes C típuskonverziós rendszert.

### 12.1 Nincs általános implicit kevert típusú aritmetika

A műveletekhez egész operandusok szükségesek, de a fordító nem épít C-s integer promotion rendszert. A legegyszerűbb szabály: ugyanazon számításban használjunk tudatosan azonos típusokat.

### 12.2 Literálok

A számliterál AST-szinten `u16`, de közvetlenül hozzárendelhető `i8/u8/i16/u16` célhoz.

### 12.3 Pointertípus

A pointerparaméter elemtípusának egyeznie kell:

```c
fn first(u16* p) -> u16 {
    return p[0];
}

fn main() -> u16 {
    u8 data[4];
    return first(&data[0]); // típushiba
}
```

---

## 13. Feltételes végrehajtás

```c
if (condition) {
    // ...
} else {
    // ...
}
```

`else if` támogatott parser-cukorként:

```c
if (x == 0) {
    return 0;
} else if (x == 1) {
    return 1;
} else {
    return 2;
}
```

A feltétel bármely skalár érték lehet (`bool`, integer vagy pointer). A nulla hamis, a nem nulla igaz. Ez vezérlési feltételként engedélyezett; ettől még egész érték nem rendelhető automatikusan `bool` változóhoz.

---

## 14. Ciklus

A C-Lite egyetlen ciklusszerkezete a `while`:

```c
u16 i = 0;
while (i < 10) {
    i = i + 1;
}
```

Nincs `for`, `do/while` vagy `goto`.

### 14.1 `break`

```c
while (true) {
    if (ready) {
        break;
    }
}
```

### 14.2 `continue`

```c
while (i < n) {
    i = i + 1;
    if (i == 2) {
        continue;
    }
    sum = sum + i;
}
```

`break` és `continue` csak cikluson belül érvényes.

---

## 15. Nyers memória és MMIO

Az architektúrafüggetlen assembly-jelleg egyik fontos része a közvetlen memóriaelérés.

### 15.1 Normál memória

```c
u8 a = load8(0x1000);
u16 b = load16(0x2000);

store8(0x1000, a);
store16(0x2000, b);
```

Szignatúrák logikailag:

```text
load8(u16 address) -> u8
load16(u16 address) -> u16
store8(u16 address, u8 value) -> void
store16(u16 address, u16 value) -> void
```

### 15.2 Volatile/MMIO

```c
vstore8(0xff00, 65);
u8 status = vload8(0xff01);
```

Elérhető:

```text
vload8
vload16
vstore8
vstore16
```

A `v` változat azt jelzi, hogy a hozzáférés MMIO/volatile jellegű. A target backend ennek megfelelő memóriautasítást használhat.

A konkrét platform MMIO címeit a platformdokumentáció tartalmazza; a C-Lite nyelv maga nem hardcode-olja az eszközöket.

---

## 16. Include

```c
include "math.cl";
```

Az include szándékosan egyszerű, szöveges include-once mechanizmus.

Keresési sorrend:

1. az aktuális forrásfájl könyvtára;
2. a `-I` kapcsolókkal megadott könyvtárak.

Példa:

```sh
svm-clite -I svm_clite/lib --target register program.cl
```

Ugyanaz a fájl egy fordítás során legfeljebb egyszer kerül beillesztésre. Ciklikus include hiba.

Nincs:

- macro-preprocessor;
- `#define`;
- feltételes fordítás;
- include guard szintaxis.

Az include-once miatt include guardra nincs szükség.

---

## 17. Kis standard library

A standard library maga is C-Lite forrás, így tanulmányozható és módosítható.

### `memory.cl`

```text
mem_zero
memcpy
memcmp
```

### `string.cl`

```text
strlen
strcmp
```

A string nullával lezárt `u8` tömb/pointer.

### `math.cl`

```text
min_u16
max_u16
abs_i16
gcd_u16
```

### `convert.cl`

```text
hex_digit
u16_to_hex
```

### `crc.cl`

```text
crc8
```

Használat:

```c
include "math.cl";

fn main() -> u16 {
    return gcd_u16(84, 30);
}
```

---

## 18. CLIR – az architektúrafüggetlen assembly

A C-Lite belső köztes nyelve a CLIR 0.1. Ez oktatási céllal megtekinthető:

```sh
svm-clite --emit ir program.cl
```

Példa C-Lite:

```c
u16 x = a + b;
```

A hozzá hasonló CLIR:

```text
load.u16 %0, a
load.u16 %1, b
add.u16 %2, %0, %1
store.u16 x, %2
```

A `%0`, `%1`, ... virtuális ideiglenes értékek. Nem valódi CPU-regiszterek.

### 18.1 Fontos CLIR műveletek

```text
const.T
load.T
store.T
addr
index
loadmem.T
storemem.T
loadmemv.T
storememv.T

add.T sub.T mul.T div.T mod.T
and.T or.T xor.T shl.T shr.T
neg.T not.T

eq.T ne.T lt.T le.T gt.T ge.T

jz
jmp
call
ret
```

A részletes specifikáció: `CLIR_0_1_HU.md`.

---

## 19. Hogyan fordul le a strukturált vezérlés?

### 19.1 `if`

C-Lite:

```c
if (a < b) {
    x = 1;
} else {
    x = 2;
}
```

Elvi CLIR:

```text
load.u16 %0, a
load.u16 %1, b
lt.u16 %2, %0, %1
jz %2, else_0
const.u16 %3, 1
store.u16 x, %3
jmp endif_1
else_0:
const.u16 %4, 2
store.u16 x, %4
endif_1:
```

### 19.2 `while`

C-Lite:

```c
while (i < n) {
    i = i + 1;
}
```

Elvi CLIR:

```text
while_test_0:
load.u16 %0, i
load.u16 %1, n
lt.u16 %2, %0, %1
jz %2, while_end_1
load.u16 %3, i
const.u16 %4, 1
add.u16 %5, %3, %4
store.u16 i, %5
jmp while_test_0
while_end_1:
```

Ezért nevezhető a C-Lite „strukturált assemblynek”: a strukturált forrás néhány egyszerű ugrássá bomlik.

---

## 20. Generált assembly és `.proc`

A C-Lite target assemblyt generál, amely használhat `.proc/.endproc` blokkokat. A C-Lite nem dönti el, mely eljárások használtak.

A külső `svm-asm`:

- feldolgozza az assembly include-okat;
- feldolgozza a `.equ` konstansokat;
- elemzi a `.proc/.endproc` kapcsolatokat;
- nem építi be a nem elérhető eljárásokat;
- elkészíti a binárist.

Ez az assembler feladata, nem C-Lite optimalizálás.

---

## 21. Hibák és `--check`

A legegyszerűbb ellenőrzési ciklus:

```sh
svm-clite --check program.cl
```

Ez nem generál assemblyt. Ellenőrzi többek között:

- lexer/parser hibákat;
- ismeretlen típust vagy változót;
- duplikált neveket;
- hibás pointertípust;
- tömb konstans indexét;
- `break/continue` helyét;
- függvény argumentumszámot és típust;
- visszatérési típust;
- közvetlen vagy kölcsönös rekurziót;
- konstans nullával osztást/modulót.

A lexer sor/oszlop pozíciót ad. Include-hibánál fájl és sorszám is megjelenik.

---

## 22. Teljes példa: tömb összegzése

```c
fn sum(u16* data, u16 count) -> u16 {
    u16 i = 0;
    u16 result = 0;

    while (i < count) {
        result = result + data[i];
        i = i + 1;
    }

    return result;
}

fn main() -> u16 {
    u16 values[4];

    values[0] = 10;
    values[1] = 20;
    values[2] = 30;
    values[3] = 40;

    return sum(&values[0], 4);
}
```

Fordítás mind a kilenc target egyikére csak a kapcsolóban tér el:

```sh
svm-clite --target register sum.cl
svm-clite --target stack sum.cl
svm-clite --target accumulator sum.cl
svm-clite --target memreg sum.cl
svm-clite --target loadstore sum.cl
svm-clite --target regmem sum.cl
svm-clite --target memory2memory sum.cl
svm-clite --target belt sum.cl
svm-clite --target tta sum.cl
```

A `.cl` forrást nem kell módosítani.

---

## 23. Teljes példa: MMIO + logika

```c
fn main() -> u16 {
    u8 status = vload8(0xff01);
    bool ready = status != 0;

    if (ready) {
        vstore8(0xff00, 65);
        return 1;
    }

    return 0;
}
```

A programban nincs Register/Stack/Accumulator specifikus részlet.

---

## 24. Tudatosan nem támogatott C-elemek

Az 1.0 vonalban szándékosan nincs:

```text
for
do/while
switch
goto
struct
union
enum
typedef
macro/#define
++ --
+= -= stb.
?:
&& || !
function pointer
pointer-to-pointer
void*
varargs
malloc/free
rekurzió
implicit C integer promotion rendszer
optimalizáló
SSA
regiszterallokátor
```

Ezek hiánya nem átmeneti hiánylista, hanem a nyelv egyszerűségi céljának része. Új elem csak akkor indokolt, ha néhány meglévő CLIR műveletre közvetlenül és könnyen lebontható.

---

## 25. Ajánlott programozási stílus

A C-Lite-ban célszerű:

- rövid, egyszerű függvényeket írni;
- explicit típusokat használni;
- `while` ciklust egyszerű feltétellel használni;
- tömböt pointer + elemszám párral átadni;
- az MMIO-t `vload*`/`vstore*` primitíveken keresztül kezelni;
- bonyolult kifejezés helyett több egyszerű lépést írni;
- a CLIR kimenetet tanulásra és hibakeresésre használni.

Például a nagyon tömör kifejezés helyett:

```c
result = (a + b) * (c - d) ^ mask;
```

oktatási és alacsony szintű kódnál sokszor átláthatóbb:

```c
u16 x = a + b;
u16 y = c - d;
u16 z = x * y;
result = z ^ mask;
```

A generált kód nem optimalizált, ezért a forrás szerkezete jól követhető marad.

---

## 26. További dokumentáció

- `LANGUAGE_HU.md` – tömör nyelvi referencia
- `CLIR_0_1_HU.md` – CLIR 0.1 specifikáció
- `CODEGEN_HU.md` – kódgenerálási modell
- `LEARNING_HU.md` – lépésenkénti tanulási útvonal
- `DESIGN_RULES_HU.md` – tervezési szabályok
- `ONE_ZERO_SCOPE_HU.md` – az 1.0 tudatos határa
- `STDLIB_HU.md` – kis standard library
- `examples/` – fordítható példaprogramok

A C-Lite megértéséhez ajánlott sorrend:

```text
PROGRAMMING_MANUAL_HU.md
    ↓
LEARNING_HU.md + examples/
    ↓
CLIR_0_1_HU.md
    ↓
CODEGEN_HU.md
    ↓
konkrét target assembly dokumentáció
```
