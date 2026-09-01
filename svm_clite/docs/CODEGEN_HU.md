# C-Lite kódgenerálás

A C-Lite fordító szándékosan egyszerű. Nem használ másik C fordítót, nem építi magába az assemblert és nem optimalizál.

A fordítási út:

```text
C-Lite forrás
    ↓
lexer
    ↓
parser
    ↓
szemantikai ellenőrzés
    ↓
CLIR 0.1
    ├→ Register ASM
    ├→ Stack ASM
    ├→ Accumulator ASM
    ├→ MemReg ASM
    ├→ LoadStore ASM
    ├→ RegMem ASM
    ├→ Memory-to-Memory ASM
    ├→ Belt ASM
    └→ TTA ASM
```

A CLIR az egyetlen architektúrafüggetlen assembly-réteg. Nem cél egy második, regiszteres pszeudo-gép ráerőltetése minden ISA-ra.

## Stack backend

A Stack backend közvetlenül CLIR-ből generál Stack16 assemblyt. A `%temp` értékek a VM adatvermében élnek, ezért például:

```text
load.u16 %0, a
load.u16 %1, b
mul.u16 %2, %0, %1
const.u16 %3, 3
add.u16 %4, %2, %3
ret %4
```

természetesen ilyen jellegű kóddá válik:

```asm
0x8000 @
0x8002 @
MUL
3
ADD
RET
```

Nincs R0..R7 emuláció és nincs statikus memóriahely minden CLIR temphez. A C-Lite valódi lokális/globális változói továbbra is statikus memóriát kaphatnak, mert a nyelv szándékosan tiltja a rekurziót.

## Kilenc saját backend

Mind a kilenc target közvetlenül CLIR-ből generál saját assemblyt. Nincs canonical vagy más közös CPU-modell. Közös csak a targetfüggetlen CLIR-adatelrendezés lehet: változók, paraméterek és szükség esetén temp-ek statikus címei. A Stack a VM adatvermét, az Accumulator az A/X modellt, a MemReg a W/file-regiszter modellt, a Memory2Memory memóriaoperandusokat, a Belt a belt értékeket, a TTA pedig közvetlen transportokat használ.

## Optimalizálás nincs

Nincs konstanshajtás, SSA, regiszterallokáció, dead-code elimináció, inlining vagy utasítás-átrendezés. A természetes target-leképezés nem optimalizáló passz.

## Az assembler külön program

A `svm-clite` target ASM fájlt ír. A binárist külön `svm-asm` készíti. A `.proc/.endproc`, assembly include és a nem használt procedure eltávolítása az assembler felelőssége.
