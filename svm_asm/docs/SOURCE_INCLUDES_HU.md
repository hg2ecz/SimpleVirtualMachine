# Assembly forrásfájl include támogatás

Az `svm-asm` egyszerű, preprocesszor nélküli forrás-include támogatást ad. A cél CPU-célonként közös assembly rutinok és konstansfájlok használata úgy, hogy a fordítás továbbra is egyetlen fordítási egységet állítson elő.

## Szintaxis

```asm
.include "lib/io.asm"
```

A behúzott fájl ugyanannak az assembly fordítási egységnek a része. Az ISA-k eltérő assembly nyelve miatt az assembly könyvtárakat célszerű célarchitektúránként külön könyvtárban tartani.

## Fájlkeresés

Relatív névnél a keresési sorrend:

1. az include-ot tartalmazó fájl saját könyvtára;
2. a parancssori `-I` könyvtárak, a megadás sorrendjében.

Példa:

```sh
svm-asm -I asm-lib register src/main.asm out.svm
```

A `-Idir` összevont alak is támogatott.

## Rekurzív include és include-once

Az include rekurzív: egy könyvtárfájl további fájlokat is behúzhat. Egy kanonikus fájl ugyanazon fordításon belül csak egyszer kerül be. A maximális include-mélység 64 fájl. A gyémánt alakú ismételt include automatikusan egyszeresre redukálódik; valódi ciklikus include esetén az assembler hibát jelez.

## Tudatos korlátozások

- nincs makrófeldolgozás;
- nincs feltételes include;
- nincs külön linkelési fázis;
- minden behúzott definíció ugyanahhoz az egy fordítási egységhez tartozik.
