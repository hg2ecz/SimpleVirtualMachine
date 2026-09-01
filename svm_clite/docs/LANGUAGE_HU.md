# SVM C-Lite 1.0 nyelv

Az SVM C-Lite egy szándékosan kicsi, strukturált, **C-szerű** nyelv a SimpleVirtualMachine architektúrákhoz. A cél nem a C szabvány másolása, hanem az, hogy assembly ismerete nélkül lehessen egyszerű algoritmusokat, ciklusokat, függvényeket, tömböket és pointeres memóriafeldolgozást írni.

A nyelv több helyről vesz át jó ötleteket. A változó-, pointer- és tömbdeklaráció C-szerű, a kifejezések és a memória-szemlélet is C-közeli. A függvényfej viszont szándékosan egyszerűbb, Rustból ismerős `fn ... -> típus` formát használ. A C-Lite ezért sem C-, sem Rust-részhalmaz.

## Példa

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

Ugyanez a forrás Register, Stack, Accumulator, MemReg, Load/Store, Register-Memory, Memory-to-Memory, Belt és TTA targetre is fordítható.

## Típusok

A nyelvi mag:

```text
bool
i8
u8
i16
u16
void
```

Egyetlen pointer-szint támogatott:

```c
u16* p;
u8* bytes;
```

Nincs `void*`, `u16**`, function pointer vagy összetett C deklarátor.


## Logikai típus

```c
bool ready = true;
bool done = false;

fn less(u16 a, u16 b) -> bool {
    return a < b;
}
```

Az összehasonlítások `bool` értéket adnak. A `bool` memóriában egy byte és mindig 0 vagy 1. `bool flags[16];` ezért 16 byte; nincs bitcsomagolás. Egész szám nem konvertálódik implicit módon `bool`-lá.

## Változók

Lokális skalár:

```c
u16 x;
u16 y = 10;
u8 ch = 65;
```

Globális skalár:

```c
u16 counter;
u16 mode = 1;
```

A C-Lite globális inicializálója csak egyszerű egész konstans lehet.

## Fix tömb

A kanonikus tömbszintaxis C-szerű:

```c
u16 values[4];
u8 buffer[256];
i16 samples[128];
```

A hossz fordítási idejű konstans, nem lehet nulla. Beágyazott tömb és pointertömb nincs.

Olvasás és írás:

```c
values[0] = 10;
u16 x = values[0];
```

Konstans indexnél a frontend ellenőrzi a határt. `values[4]` hibás egy `u16 values[4];` tömbnél.

## Pointer

```c
u16 x = 12;
u16* p = &x;
*p = 20;
```

Tömb elemének címe:

```c
u16 values[4];
foo(&values[0]);
```

Pointer indexelés:

```c
p[i]
```

A fordító automatikusan figyelembe veszi az elem méretét: `u8*` egy byte-tal, `u16*` két byte-tal lép. A programozónak nem kell assembly címzést vagy regisztereket ismernie.

## Függvény

```c
fn add(u16 a, u16 b) -> u16 {
    return a + b;
}
```

Pointerparaméter:

```c
fn clear(u8* data, u16 count) {
    u16 i = 0;
    while (i < count) {
        data[i] = 0;
        i = i + 1;
    }
}
```

Ha nincs `-> típus`, a visszatérés `void`.

## Vezérlés

A minimális mag:

```text
if / else
while
break
continue
return
```

A `while` az egyetlen ciklusszerkezet. Ez szándékos: minden ismétlés ugyanarra az egyszerű teszt–törzs–ugrás modellre épül.

## Operátorok

```text
+ - * / %
& | ^ ~
<< >>
== != < <= > >=
```

## Include

```c
include "math.cl";
```

Az include egyszerű forrásszintű beillesztés. Keresés: az aktuális fájl könyvtára, majd a `-I` útvonalak. Ciklikus include hiba.

## Globális memória

Globális fix tömb használható például framebufferhez vagy kommunikációs bufferhez:

```c
u8 framebuffer[32000];
u16 frame_counter;

fn main() -> u16 {
    framebuffer[0] = 1;
    frame_counter = frame_counter + 1;
    return frame_counter;
}
```


## Tudatosan hiányzó elemek

A nyelv egyszerűsége fontosabb a C-kompatibilitásnál. A nyelvben szándékosan nincs:

- `struct`, `union`, `enum`, `typedef`;
- makró/preprocesszor;
- többes pointer;
- function pointer;
- dinamikus memória;
- varargs;
- rekurzióra építő ABI;
- összetett C deklarátor;
- tömb inicializáló lista.

A cél: **assembly szintű feladatok strukturált leírása assembly szintaxis nélkül**.

## Egyszerű névtér

A C-Lite szándékosan nem enged névárnyékolást. Egy lokális változó vagy paraméter nem használhatja ugyanazt a nevet, mint egy látható globális vagy ugyanazon függvény másik változója. Ez eltér a C-től, de egyszerűbbé teszi a hibakeresést és a statikus memória-kiosztást.

## Architektúrafüggetlen assembly-szintű IR

A nyelv mögött megjelent egy kicsi, targetfüggetlen IR. `svm-clite --emit ir program.cl` formában megtekinthető. Az IR virtuális temp-eket, `load/store`, `loadmem/storemem`, aritmetikai, ugrás-, hívás- és visszatérési műveleteket használ, de semmilyen konkrét ISA-regisztert vagy stack/belt/TTA részletet nem tartalmaz. Részletesen: `CLIR_0_1_HU.md`.

### Nyers memória és MMIO

Az assembly-közeli feladatokhoz a C-Lite közvetlen targetfüggetlen memória-primitíveket biztosít:

```c
u8 x = load8(0x1000);
u16 y = load16(0x2000);
store8(0x1000, x);
store16(0x2000, y);
```

MMIO-hoz a volatile változat használható:

```c
vstore8(0xff00, 65);
u8 status = vload8(0xff01);
```

Elérhető: `load8`, `load16`, `store8`, `store16`, `vload8`, `vload16`, `vstore8`, `vstore16`. Ezek a backendben a kiválasztott architektúra megfelelő memóriautasításaira fordulnak; a C-Lite forrásban nincs szükség regiszter- vagy portszintaxisra.

### Nincs rekurzió

A C-Lite a közvetlen és kölcsönös rekurziót fordítási hibaként kezeli. Ez szándékos: egyszerű statikus lokális/paraméter kiosztást tesz lehetővé, általános stack frame nélkül.

## Egyszerű vezérlés és konstansok

A `while` az egyetlen ciklusszerkezet. Az `else if` csak parser-cukor, belül egymásba ágyazott `if`.

### Számliterálok

A három assembly-közeli forma közvetlenül használható:

```c
u16 a = 1234;        // decimális
u16 b = 0x04d2;      // hexadecimális
u16 c = 0b00000100;  // bináris
```

Mindegyik 16 bites literál. A túl nagy literál fordítási hiba.

### Nincs optimalizálás

A C-Lite szándékosan nem optimalizál, konstanshajtást sem végez. A `2 + 3 * 4` műveletsor külön CLIR utasításokként marad meg. Ez egyszerűbbé és oktathatóbbá teszi a fordítót.



## Include és kommentek

Az include egyszerű, szöveges beillesztés:

```c
include "math.cl";
```

Nincs makró-preprocesszor. Az include külön soron áll. Kommentként `// ...` és `/* ... */` használható. A kommentek nem kerülnek az AST-ba és nem befolyásolják a generált kódot.

## Egyszerűségi szabályok 1.0-hoz

Az include egyszerű textual include-once: ugyanaz a fájl egy fordítás során csak egyszer kerül be, a ciklikus include hiba. Macro-rendszer nincs.

A lokális változók blokkosan láthatók, de egy függvényen belül minden lokális névnek egyedinek kell lennie. Ennek oka szándékosan egyszerű: a közvetlen backend minden lokálisnak egyetlen statikus memóriahelyet foglal, így nincs scope-alapú slot-újrahasznosítás.

A fordító nem optimalizál. A forrás műveletei CLIR műveletekként megmaradnak, majd mechanikusan assemblyre fordulnak.
