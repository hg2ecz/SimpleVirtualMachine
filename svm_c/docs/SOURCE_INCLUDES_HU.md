# SVM-C forrásfájl include támogatás

Az SVM-C egyszerű, preprocesszor nélküli forrás-include támogatást ad saját függvénykönyvtárak és közös konstans-/rutinfájlok használatához. A fordítás továbbra is egyetlen fordítási egységet állít elő.

## Szintaxis

```c
include "lib/math.sc";
include "lib/console.sc";

u16 main() {
    return add3(5);
}
```

Az `include` nem C-preprocesszor direktíva, ezért nincs `#include`, makró, `#define` vagy feltételes fordítás. A behúzott forrás a fő fájl részeként kerül lexikális és szintaktikai feldolgozásra.

## Fájlkeresés

Relatív névnél a keresési sorrend:

1. az include-ot tartalmazó fájl saját könyvtára;
2. a parancssori `-I` könyvtárak, a megadás sorrendjében.

Példák:

```sh
svm-c --target register -I lib src/main.sc out.svm
svm-c --target register -O2 -I lib src/main.sc out.svm
```

A `-Idir` összevont alak is támogatott.

## Rekurzív include és include-once

Az include rekurzív. Egy kanonikus fájl ugyanazon fordításon belül csak egyszer kerül be. A maximális include-mélység 64 fájl. A gyémánt alakú ismételt include automatikusan egyszeresre redukálódik; valódi ciklikus include esetén a fordító hibát jelez. Hiányzó fájlnál a fordító jelzi az include helyét és az include-láncot.

## Kapcsolat az optimalizálással

Az include csak forrásszintű láthatóságot ad. `-O1`, `-O2` és `-Os` alatt a `main()`-ből tranzitívan nem elérhető függvények már a statikus layout előtt kiesnek, ezért sem generált assemblyt, sem compiler-owned statikus RAM-ot nem foglalnak. `-O0` és `svm-c-unopt-only` minden beolvasott függvényt eljuttat a C-szintű kódgenerálásig és `--emit asm` esetén meg is mutatja őket; bináris készítéskor azonban a közös assembler procedure-GC minden optimalizációs szinten eltávolítja az el nem érhető `.proc` blokkokat, ezért azok gépi kódhelyet nem foglalnak.

## Tudatos korlátozások

- nincs `#include`;
- nincs makrófeldolgozás;
- nincs feltételes include;
- nincs külön linkelési fázis;
- minden behúzott definíció ugyanahhoz az egy fordítási egységhez tartozik;
- azonos nevű globális vagy függvénydefiníció fordítási hiba.
