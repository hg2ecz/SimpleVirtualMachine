# SVM C-Lite 1.0 – rögzített egyszerűségi határ

A C-Lite célja nem a C kompatibilitás. A cél egy kicsi, strukturált, architektúrafüggetlen assembly-szerű nyelv, amelyhez nem kell a kilenc SVM ISA assembly szintaxisát megtanulni.

## 1.0-ban benne van

- `bool`, `i8`, `u8`, `i16`, `u16`, `void`
- egy pointer-szint: `bool*`, `u8*`, `u16*`, `i8*`, `i16*`
- fix tömb: `u16 values[4];`
- globális és lokális skalár/tömb
- `fn`, paraméterek, `return`
- `if/else`
- egyetlen ciklus: `while`
- `break`, `continue`
- `+ - * / %`
- `& | ^ ~ << >>`
- összehasonlítások
- `&x`, `*p`, `p[i]`
- `load8/load16/store8/store16`
- `vload8/vload16/vstore8/vstore16`
- textual `include "file.cl";`, include-once és ciklusdetektálás
- `//` és `/* ... */` komment
- `--check`, `--emit ir`, `--emit asm`
- opcionális külső `svm-asm` hívás `--assemble` kapcsolóval

## Tudatosan nincs

- optimizer vagy konstanshajtás
- `for`, `switch`, `goto`
- `struct`, `union`, `enum`, `typedef`
- macro-preprocessor
- `++`, `--`, `+=` és hasonló szintaktikai rövidítések
- pointer-to-pointer, `void*`, function pointer
- dinamikus memória
- rekurzió
- regiszterallokátor, SSA, linker vagy saját machine-code encoder

## Fordítási modell

```text
C-Lite
  -> parser + szemantikai ellenőrzés
  -> CLIR 0.1
  -> egyszerű közvetlen target ASM
  -> külső svm-asm
```

A C-Lite compiler nem távolít el nem használt függvényt. `.proc/.endproc` blokkokat generál, a procedure-GC az assembler feladata.

## Kódminőség

Az 1.0 célja a helyes és kiszámítható, nem az optimális kód. Egy forrásművelet CLIR műveletekre, azok pedig mechanikusan assemblyre bomlanak. Ez oktathatóvá és könnyen hibakereshetővé teszi a fordítót.

## Bool egyszerűségi szabály

A `bool` memóriában 1 byte, kizárólag 0/1 reprezentációval. Nincs bitpacking. Az összehasonlítások `bool` eredményt adnak. Ez nem vezet be optimalizálót; a backend jelenleg mechanikusan materializálja a 0/1 értéket.

Lásd: `DESIGN_RULES_HU.md`.
