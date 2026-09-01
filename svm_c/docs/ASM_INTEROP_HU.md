# SVM-C és assembly együttműködés

Az SVM-C targetfüggetlen C forrásból target-specifikus assembly modult tud meghívni.

## C oldal

```c
asm_include "crc_fast.asm";
extern asm u16 crc16_fast(u16 addr, u16 len);

u16 main() {
    return crc16_fast(0x2000, 128);
}
```

Az `asm_include` logikai modulnevet ad meg. Nem kell beleírni az architektúra nevét.
Az `extern asm` csak deklaráció; C függvénytörzs nem tartozik hozzá.

## Többarchitektúrás könyvtárszerkezet

Projektben az ajánlott elrendezés:

```text
program.sc
asm/
  register/crc_fast.asm
  stack/crc_fast.asm
  accumulator/crc_fast.asm
  memreg/crc_fast.asm
  loadstore/crc_fast.asm
  regmem/crc_fast.asm
  memory2memory/crc_fast.asm
  belt/crc_fast.asm
  tta/crc_fast.asm
```

A `svm-c --target stack program.sc` automatikusan az `asm/stack/` könyvtárat keresi. A generált assembly alapkönyvtára (a `.sc` fájl mappája) közvetlen fájl-fallbackként is használható. A `-I DIR` esetén előbb `DIR/<target>/`, majd `DIR/` kerül a keresési listába. Végül a beépített `svm_asm/lib/<target>/` könyvtár következik. Többarchitektúrás projektnél az `asm/<target>/` elrendezés az ajánlott; a közvetlen fallback inkább egytargetes vagy lokális modulhoz való.

## Stabil C-ASM bridge ABI

A fordító minden `extern asm` deklarációhoz egy target-specifikus C wrapper-t generál. Az assembly implementáció neve:

```text
__asm_<C-fuggvenynev>
```

A paramétereket a wrapper statikus memóriahelyekre teszi. Az assembler számára a fordító `.equ` szimbólumokat ad:

```text
__cabi_<fuggveny>_<parameter>
__cabi_<fuggveny>_return
```

Például:

```asm
.proc __asm_crc16_fast
    ; bemenet:
    ;   __cabi_crc16_fast_addr
    ;   __cabi_crc16_fast_len
    ; eredményt ide kell írni:
    ;   __cabi_crc16_fast_return
    ...
    RET
.endproc
```

`void` visszatérésű függvénynél nincs `_return` hely. A bridge-hely mérete mindig a deklarált C típus mérete: `i8/u8/bool` 1 byte, `i16/u16` 2 byte. A jelenlegi C nyelvi szabály szerint széles (`i32/u32/i64/u64`) érték nem adható át vagy vissza közvetlenül függvényparaméterként; ezekhez `u16` címet kell átadni, ugyanúgy mint a C könyvtári wide-aritmetikánál. A többbyte-os adatok a VM normál little-endian memóriaábrázolását követik.

Ez a memória-alapú bridge szándékos: a Register, Stack, Accumulator, Belt, TTA stb. natív hívási ABI-ja eltér, de az assembly modul szerzője minden targeten ugyanazt a logikai paramétermodellt kapja. A generált wrapper alakít át a C ABI és a bridge ABI között.

## Procedure-GC

Az `asm_include` nem jelenti azt, hogy az egész ASM modul bekerül a programba. A wrapper közvetlenül a `__asm_<név>` eljárást hívja, ezért a procedure-GC csak ezt és tranzitív függőségeit tartja meg.

## `--emit asm`

`--emit asm` esetén a kimenet szándékosan megtartja a `.include` direktívát, a C wrapper-t és a `__cabi_*` szimbólumokat. `--emit bin` esetén a C driver kibontja az assembly include-okat, feldolgozza a `.equ` konstansokat, majd lefuttatja a procedure-GC-t.

## Példa

A `svm_c/examples/extern_asm.sc` ugyanaz mind a kilenc targetre. Az implementációk a `svm_c/examples/asm/<target>/interop_demo.asm` fájlokban vannak.

## Fenntartott nevek

A `__asm_` és `__cabi_` prefixek a fordító C/ASM bridge-éhez vannak fenntartva. C globális vagy függvénynév nem kezdődhet ezekkel a prefixekkel. Az assembly implementációban a `__asm_<C-nev>` belépési pontot és a fordító által generált `__cabi_*` szimbólumokat kell használni, de új felhasználói szimbólumot ne nevezzünk el ezekkel a prefixekkel.

## Paramétertípusok

Az `extern asm` ugyanazokat a skalár paraméter- és visszatérési típuskorlátokat követi, mint a normál SVM-C függvények. A bridge minden paraméterhez a C típus méretének megfelelő memóriahelyet foglal; az ASM rutin az adott típusnak megfelelő 8 vagy 16 bites művelettel olvassa/írja azt. Széles (`i32/u32/i64/u64`) értéket továbbra is címparaméterrel célszerű átadni. `void` visszatérésnél `__cabi_<fuggveny>_return` szimbólum nem keletkezik.
