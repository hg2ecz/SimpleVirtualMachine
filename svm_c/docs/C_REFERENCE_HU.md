# SVM-C nyelvi referencia

Ez a dokumentum a **jelenleg ténylegesen implementált** SVM-C forrásnyelvet írja le. Az SVM-C C-szerű, freestanding rendszerprogramozási nyelv; nem ANSI/ISO C.

## 1. Típusok

| Típus | Méret | Jelentés |
|---|---:|---|
| `bool` | 1 byte | logikai tárolás; 0 hamis, nem nulla igaz |
| `i8` | 1 byte | 8 bites kétkomplementes signed bitminta |
| `u8` | 1 byte | 8 bites unsigned |
| `i16` / `int` | 2 byte | 16 bites signed/kétkomplementes érték |
| `u16` | 2 byte | 16 bites unsigned |
| `i32` / `long` | 4 byte | cím-alapú többwordös objektum |
| `u32` | 4 byte | cím-alapú többwordös objektum |
| `i64` | 8 byte | 32×32 signed teljes szorzat tárolója |
| `u64` | 8 byte | 32×32 unsigned teljes szorzat tárolója |
| `void` | - | csak függvény-visszatérési típus |

A CPU-k 16 bites integer gépek. A `bool/i8/u8/i16/u16` natív skalár. Az `i32/u32/i64/u64` **address-only wide objektum**: közvetlen értékkifejezésként, by-value paraméterként vagy függvény-visszatérési típusként nem használható; a wide integer és `f32` könyvtári rutinok `&objektum` címeket kapnak. Az `i64/u64` típushoz nincs általános 64 bites aritmetikai API; elsődleges szerepük a 32×32 teljes szorzat eredményének tárolása.

A részletes numerikus modell: `NUMERIC_TYPES_HU.md`.

## 2. Literálok és megjegyzések

Egész literál:

```c
1234
0x1234
0XABCD
```

Az értéknek `0..0xFFFF` tartományba kell férnie.

Megjegyzések:

```c
// egysoros
/* több soros */
```

Karakterliterál (`'A'`) nincs. String literál csak a `puts("...")` speciális argumentumaként használható.

Támogatott string escape-ek: `\n`, `\r`, `\t`, `\0`, `\\`, `\"`.

## 3. Változók és statikus tárolás

Globális és függvényen belüli változók támogatottak:

```c
u16 counter;
u8 mode = 3;

u16 main() {
    u16 x = 10;
    return x;
}
```

A globális inicializáló csak közvetlen numerikus konstans lehet. A lokális inicializáló általános kifejezés lehet.

A lokális változók és paraméterek **statikusan lefoglalt memóriában** élnek, nem stack frame-ben. A hívó a callee paraméterhelyeire írja az argumentumokat a `CALL` előtt; a Stack backend természetes vermes átadást használ. Emiatt nincs külön négyelemű paraméterkorlát. A jelenlegi ABI továbbra sem rekurzív és nem reentráns.

A statikus allocator alapértelmezésben először `0x0000..0x00EF`, majd `0xE000..0xFAFF` területet használ; `0x00F0..0xDFFF` nem kerül normál statikus objektumok kiosztására. Két célarchitektúrának van további foglalása: MemReg esetén `0x000E..0x000F` compiler-owned hot scratch, Memory-to-Memory esetén pedig a felhasználói/statikus kiosztás `0x0020`-tól indul, mert `0x0000..0x001F` compiler-owned scratch. A C programkép `0x0100..0xDFFF` között nőhet; `0xE000..0xFAFF` a statikus overflow-terület, `0xFB00..0xFEFF` a runtime stack konvenció, `0xFF00..0xFFFF` MMIO.

Egy függvényen belül nincs blokk-szintű névárnyékolás: ugyanaz a lokális név belső blokkban sem deklarálható újra.

## 4. Fix méretű tömbök

Támogatott:

```c
u8 bytes[32];
u16 words[16];

bytes[i] = 7;
x = bytes[i];
words[2] = 0x1234;
```

Szabályok:

- méret csak pozitív numerikus konstans;
- fix tömb bármely tárolási típusból deklarálható; wide tömbelem értékként nem tölthető be, csak cím-alapú könyvtári művelettel kezelhető;
- globális és lokális tömb is lehet;
- nincs tömbinicializáló;
- nincs tömbparaméter;
- dinamikus indexnél nincs futásidejű határellenőrzés;
- fordításkor ismert, tartományon kívüli konstans index hiba;
- a tömb neve értékként a tömb 16 bites báziscímét adja.

A tömbnév báziscímként való használata egyszerű, SVM-C-specifikus szabály; nem teljes ANSI C array-to-pointer decay modell.

## 5. Értékadás, `++/--`, összetett értékadás

Egyszerű értékadás:

```c
x = y + 1;
a[i] = x;
```

Önálló utasításként skaláron és tömbelemen támogatott:

```c
x++;
x--;
x += 2;
x -= 2;
x *= 3;
x /= 3;
x %= 7;
x &= mask;
x |= mask;
x ^= mask;
x <<= 1;
x >>= 1;

a[i]++;
a[i] += 2;
```

Az összetett tömbelem-művelet indexének mellékhatásmentesnek kell lennie. Ez jó:

```c
a[i + 1] += 2;
```

Ez jelenleg tiltott:

```c
a[getc()] += 2;
```

mert az egyszerű lowering egyébként többször értékelhetné az indexet. A sima `a[getc()] = x;` megengedett.

Nincs prefix `++x`/`--x`, és a postfix forma nem kifejezés: `y = x++;` nem támogatott.

## 6. Kifejezések és operátor-precedencia

Erősebbtől gyengébb felé:

| Szint | Operátorok |
|---|---|
| unáris | `- ~ !` |
| szorzás | `* / %` |
| összeadás | `+ -` |
| shift | `<< >>` |
| reláció | `< > <= >=` |
| egyenlőség | `== !=` |
| bit AND | `&` |
| bit XOR | `^` |
| bit OR | `|` |
| logikai AND | `&&` |
| logikai OR | `||` |

A `&&` és `||` **rövidzáras**: a jobb oldal csak akkor fut le, ha az eredmény meghatározásához szükséges. Logikai eredmény 0 vagy 1.

Nincs `?:`, vesszőoperátor vagy általános assignment-expression.

## 7. `sizeof`

Támogatott formák:

```c
sizeof(u8)       // 1
sizeof(u16)      // 2
sizeof(int)      // 2
sizeof(x)
sizeof(buffer)
```

A zárójelben típusnév vagy **egy objektumnév** állhat. Általános kifejezés, például `sizeof(a + b)`, jelenleg nem támogatott. Tömbnél a teljes byte-méretet adja.

## 8. Vezérlési szerkezetek

### `if / else`

```c
if (x == 0) {
    y = 1;
} else {
    y = 2;
}
```

Nulla hamis, nem nulla igaz.

### `while`

```c
while (i < 100) {
    i++;
}
```

### `do ... while`

```c
do {
    i++;
} while (i < 100);
```

### `for`

```c
for (u16 i = 0; i < 10; i++) {
    sum += i;
}
```

Mindhárom fejlécmező elhagyható. Az init és step mező az SVM-C egyszerű utasításformáját használja: deklaráció, értékadás, `++/--`, compound assignment vagy kifejezésutasítás.

### `break` és `continue`

Mindhárom ciklusban támogatott. Cikluson kívül fordítási hiba.

A `continue` célja:

- `while`: feltétel újraellenőrzése;
- `for`: step, majd feltétel;
- `do...while`: feltétel.

Nincs `switch`, `goto` vagy címke.

## 9. Függvények

```c
u16 add(u16 a, u16 b) {
    return a + b;
}

void hello() {
    puts("hello");
    return;
}
```

Szabályok:

- a skalár paraméterek száma nincs mesterségesen négyre korlátozva; a gyakorlati korlát a statikus adatterület és a híváskori runtime stack kapacitása;
- paraméter nem lehet `void` és nem lehet tömb;
- külön prototype/declaration nincs;
- `void` eredmény nem használható értékként;
- a programnak `main()` függvényt kell definiálnia;
- közvetlen és közvetett rekurzió tiltott;
- variadikus függvény és function pointer nincs.

## 10. `puts()` és stringek

A jelenlegi string-támogatás tudatosan szűk:

```c
puts("Hello VT100");
```

A `puts()` pontosan egy string literált vár. A fordító a karaktereket a VT100 konzolra küldi, majd `CR` + `LF` sorvéget ír.

Nincs általános string objektum, string változó, `char *`, string pointer vagy string literalból képzett normál címérték.

## 11. Beépített függvények

### Normál 64 KiB rendszer-címtér

| Builtin | Visszatérés | Jelentés |
|---|---|---|
| `load8(addr)` | `u8` | byte olvasás |
| `load16(addr)` | `u16` | little-endian 16 bites olvasás |
| `store8(addr,val)` | `void` | byte írás |
| `store16(addr,val)` | `void` | little-endian 16 bites írás |

### Külön 16 KiB VRAM

| Builtin | Visszatérés | Jelentés |
|---|---|---|
| `vload8(addr)` | `u8` | VRAM byte olvasás |
| `vload16(addr)` | `u16` | VRAM 16 bites olvasás |
| `vstore8(addr,val)` | `void` | VRAM byte írás |
| `vstore16(addr,val)` | `void` | VRAM 16 bites írás |

### VT100 / RS-232

| Builtin | Visszatérés | Jelentés |
|---|---|---|
| `putc(ch)` | `void` | egy byte küldése |
| `puts("...")` | `void` | string literal + CR/LF |
| `getc()` | `u8` | blokkoló byte fogadás |

### Teljesítményszámlálók

| Builtin | Jelentés |
|---|---|
| `clock_lo()` / `clock_hi()` | 32 bites VM ciklusszámláló két fele |
| `instr_lo()` / `instr_hi()` | 32 bites retired-instruction számláló két fele |

### Fixed-point / DSP

| Builtin | Jelentés |
|---|---|
| `asr1(x)` | előjeles aritmetikai jobbra shift 1 bittel |
| `mul_q15(a,b)` | signed Q15xQ15, 32 bites köztes érték, Q15 visszaskálázás |

A `mul_q15()` a `-32768 * -32768` speciális esetet `0x7FFF` értékre telíti.

## 12. Tudatosan nem támogatott

- `char`, `short`, C-s `signed` kulcsszó és önálló `signed` deklarációs forma; (`long` támogatott, az `i32` aliasa);
- `float`, `double`;
- általános pointer-deklaráció, dereferálás (`*p`) és pointer-pointer műveletek; az address-only wide objektumokhoz szükséges `&objektum` címképzés támogatott;
- `struct`, `union`, `enum`, `typedef`;
- karakterliterál;
- általános string/pointer szemantika;
- tömbinicializáló, tömbparaméter, VLA;
- `switch/case/default`, `goto`;
- prefix `++/--` és értéket adó postfix `++/--`;
- `?:`, vesszőoperátor, assignment-expression;
- cast-szintaxis;
- általános `sizeof(expression)`;
- preprocessor és header rendszer;
- `static`/`extern`/linkage modell;
- variadikus függvény;
- dinamikus memóriafoglalás;
- stack-frame automatikus lokálisok, rekurzió, reentrancia.

## 13. Rövid gyakorlati példa

```c
u8 data[16];

u16 main() {
    u16 i = 0;
    u16 sum = 0;

    puts("SVM-C example");

    for (i = 0; i < sizeof(data); i++) {
        data[i] = i;
        if ((i & 1) == 0) {
            continue;
        }
        sum += data[i];
        if (sum > 40) {
            break;
        }
    }

    return sum;
}
```

## 14. Optimalizálás és unopt-only irány

A `svm-c` `-O0`, `-O1`, `-O2`, `-Os` szintjei a generált kódot módosítják, nem a nyelv szintaxisát. A külön `svm-c-unopt-only` ugyanazt a frontendet és backendeket használja, de egyáltalán nem futtat AST-optimalizáló passzt és nem fogad `-O` kapcsolót. Az optimizer belső `Inc1/Dec1/Shl1/Shr1` AST-formái **nem** forrásnyelvi operátorok.



`-O1`, `-O2` és `-Os` esetén a fordító a `main()`-ből induló közvetlen hívási gráfot tranzitívan bejárja, és a nem elérhető függvényeket még a statikus memória-kiosztás előtt eltávolítja. Emiatt ezek sem generált assemblyt, sem compiler-owned statikus RAM-helyet nem foglalnak. `-O0` és `svm-c-unopt-only` minden beolvasott függvényt eljuttat a C-szintű kódgenerálásig az oktatási összehasonlíthatóság miatt, és `--emit asm` esetén a teljes `.proc` készlet megmarad. Bináris kimenetnél viszont minden optimalizációs szinten lefut az assembler procedure-GC, ezért az el nem érhető generált eljárások gépi kódhelyet ekkor sem foglalnak. A nyelv jelenleg nem támogat függvénypointert, ezért a C-szintű direkt hívási gráf teljes.

## Forrás include

Saját könyvtárak behúzásához a fordító a `include "fajl.sc";` formát támogatja. Ez nem preprocesszor: a fájl szövege a fordítás előtt ugyanabba a fordítási egységbe kerül. Az útvonal a behúzó fájlhoz képest relatív, illetve `-I` keresési könyvtárak adhatók meg. Egy fájl fordításonként egyszer kerül be.

## 15. Célarchitektúrák és ISA-specifikus kódgenerálás

A fordító kilenc célarchitektúrát támogat: `register`, `stack`, `accumulator`, `memreg`, `loadstore`, `regmem`, `memory2memory`, `belt`, `tta`. A forrásnyelv és a szemantika közös; csak a backend kódgenerálása tér el.

A jelenlegi ISA-revízió fontos kódgenerálási szabályai:

- **Register:** az `R0..R3` compact logikai művelete az `AND`; az `XOR` teljes értékű normál ALU-művelet marad. A `SUBI` natív, mert a `C` no-borrow szemantikát meg kell őrizni. Konstans maszkolásnál a backend szükség esetén `MOVI` + compact `AND` sorozatot használ.
- **MemReg:** a `0x00..0x0F` hot file ablak egybájtos logikai gyorsítása `AND`, nem `XOR`; az XOR normál formában továbbra is elérhető. A compiler-owned 16 bites scratch `0x000E..0x000F`, ezért a gyakori ideiglenes `MOV16`, `ADD` és `AND` rövid hot kódolást kaphat.
- **Load/Store:** strict load/store modell; a `SUBI` hosszú-immediate formája saját `SUBI16` dekódot használ a helyes carry/no-borrow flag miatt. Nincs automatikus post-increment load/store.
- **Stack:** a kézi assembly/Forth programozhatóság miatt több stack-manipuláló és strukturált loop utasítás a mag ISA-ban marad. A többwordös aritmetikához csak egy minimális rejtett `C` állapot van; az összehasonlítások továbbra is stackértéket adnak.
- **Accumulator, Register-Memory, Memory-to-Memory:** a saját operandusmodelljük természetes formáit használják; hardveres floating point egyik célon sincs.

Az `f16` és `f32` aritmetika minden célon szoftveres könyvtár. A 32 bites egész és soft-float kódot az integer `ADC/SBC/MULHU/RCR1` jellegű segédprimitívek gyorsíthatják, de a CPU-k 16 bites integer gépek maradnak.

### Belt16 target

A `belt` / `belt16` target nyolc elemű (`b0..b7`) implicit eredményszalagot használ. A jelenlegi C backend a közös virtuális temporaries-t a `0x0000..0x000F` compiler-owned memóriaablakba süllyeszti, ezért a Belt C statikus objektumai `0x0020`-tól indulnak. Ez konzervatív referencia-lowering; későbbi belt-specifikus optimalizálás a rövid életű eredményeket közvetlenül a belten tarthatja. Hardveres floating point nincs.


### TTA16 target

A `tta` / `tta16` target transport-triggered kódot generál. Az ALU-műveletek explicit adatmozgatásokból állnak: az első operandus `ALU.X`-re kerül, a második a megfelelő `ALU.*` triggerportra, majd az eredmény `ALU.OUT`-ból olvasható ki. A C nyelv szemantikája nem változik; hardveres floating point nincs.

## C-first standard library modulok

Az általános hordozható algoritmusok a `svm_c/lib/` alatt C forrásként találhatók. A `stdlib.sc` umbrella include a memória-, string-, bit-, CRC/checksum-, konverziós, ring-buffer-, integer/Q15/trig-, random- és konzolsegédeket gyűjti össze. Részletes API: `STANDARD_LIBRARY_HU.md`.

A cím-alapú API-k `u16` címet fogadnak; tömb vagy objektum címe `&objektum` alakban képezhető. A compiler built-in `puts()` továbbra is csak string literált fogad; dinamikus memóriastringhez a `console.sc` `putstr(address)` rutinja használható.
