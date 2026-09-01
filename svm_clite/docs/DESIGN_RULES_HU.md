# C-Lite tervezési szabályok

A C-Lite célja nem egy kis C fordító létrehozása. A cél egy **strukturált, architektúrafüggetlen assembly nyelv**, amelyhez nem kell megtanulni a kilenc SVM ISA assembly szintaxisát.

A két elsődleges követelmény:

1. a nyelv maradjon kicsi és könnyen tanulható;
2. a fordító maradjon kicsi és könnyen átlátható.

## Új nyelvi elem csak egyszerű leképezéssel

Új elem akkor kerülhet a nyelvbe, ha néhány meglévő CLIR műveletre közvetlenül lebontható. Ha új optimizer-pass, SSA, regiszterallokátor, bonyolult típusrendszer vagy targetenként külön algoritmus kellene hozzá, akkor nem való a C-Lite-ba.

## Nincs optimalizáló

A fordító nem próbál jobb kódot keresni. A forrást mechanikusan CLIR-re, majd target assemblyre fordítja. A cél a helyesség, kiszámíthatóság és oktathatóság.

## `bool`

A `bool` egyszerű kivétel, mert közvetlenül megfelel egy logikai állapotnak:

- nyelvben: `bool`, `true`, `false`;
- CLIR-ben: `.bool` típus;
- memóriában: 1 byte, értéke 0 vagy 1;
- tömbben: egy `bool` elem egy byte;
- nincs bitpacking és nincs külön bool allocator.

Egy target backend később közvetlen flag/carry/predicate leképezést használhat, ha ez egyértelmű helyi megvalósítás. Ehhez nem vezetünk be külön optimalizáló passzt.

## Felelősségek

```text
C-Lite frontend
  lexer -> parser -> semantic check -> CLIR

C-Lite backend
  CLIR -> mechanikus target ASM

svm-asm
  include/.equ/.proc feldolgozás -> bináris
```

A C-Lite nem linkel assemblert, nem végez procedure-GC-t és nem generál közvetlen gépi kódot.

## Backend egyszerűség és kódminőség

A „közvetlen backend” nem pusztán azt jelenti, hogy nincs canonical köztes ISA. A backendnek a rövid életű CLIR tempet a target természetes állapotában kell tartania, ha ez egyszerűen megtehető. Stack gépen ez adatverem, accumulator gépen A, belten belt slot, regiszteres gépen kis fix expression-register készlet lehet. A fölösleges temp-RAM nem kívánt alapértelmezés.

Ez továbbra sem optimizer: nincs utólagos mintakeresés vagy globális elemzés, csak közvetlen target-specifikus reprezentáció és instruction selection.
