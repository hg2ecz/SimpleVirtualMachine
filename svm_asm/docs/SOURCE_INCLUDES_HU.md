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
2. a parancssori `-I` könyvtárak, a megadás sorrendjében;
3. az `svm-asm` CLI által automatikusan hozzáadott `svm_asm/lib/<target>/` beépített könyvtár.

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

## Standard targetkönyvtár

A parancssori assembler a kiválasztott architektúra könyvtárát automatikusan include-útvonalként kezeli. Így a kézi programban elegendő például:

```asm
.include "platform.asm"
.include "console.asm"
```

A `platform.asm` minden targetnél azonos szimbolikus MMIO-neveket ad a `.equ` előfeldolgozó segítségével.

## Kapcsolat a procedure-GC-vel

Az include továbbra is szövegesen egyetlen fordítási egységet hoz létre, de ez már nem jelenti azt, hogy minden behúzott rutin bekerül a binárisba. A könyvtári rutinokat `.proc/.endproc` blokkokban kell deklarálni; az include és `.equ` kifejtése után a procedure-GC csak az elérhető blokkokat hagyja meg. A konstansokat és az eljáráson kívüli globális forrásrészeket a passz nem tekinti elhagyható eljárásnak.
