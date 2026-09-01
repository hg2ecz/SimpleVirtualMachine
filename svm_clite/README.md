# svm-clite

`svm-clite` egy szándékosan kicsi, C-szerű, strukturált nyelv a SimpleVirtualMachine kilenc architektúrájához. A cél nem a teljes C, hanem egy **architektúrafüggetlen assembly** némi strukturáltsággal, target-specifikus assembly szintaxis tanulása nélkül.

## Modell

```text
C-Lite -> CLIR 0.1 -> target ASM -> külső svm-asm
```

A fordítóban nincs optimizer, SSA, regiszterallokátor, linker, beépített assembler vagy SVM-C réteg.

## Nyelvi mag

- `bool`, `i8`, `u8`, `i16`, `u16`, `void`
- `u16 x;`, `u16 values[4];`, `u16* p;`
- `fn ... -> ...`
- `if / else`
- `while`, `break`, `continue`
- fix tömb és egy pointer-szint
- függvény és paraméterátadás
- textual `include`
- `//` és `/* ... */` komment
- `load8/load16/store8/store16`
- `vload8/vload16/vstore8/vstore16` MMIO-hoz

A rekurzió tiltott. A `bool` memóriában egy byte, nincs bitcsomagolás.

## Használat

```sh
svm-clite --check program.cl
svm-clite --emit ir program.cl
svm-clite --target register program.cl
svm-asm register program.asm program.svm
```

A `--assemble` csak kényelmi wrapper egy külső `svm-asm` meghívására.

## Dokumentáció

- `docs/PROGRAMMING_MANUAL_HU.md` – teljes magyar programozói kézikönyv
- `docs/PROGRAMMING_MANUAL_EN.md` – complete English programmer's manual
- `docs/RELEASE_CHECKLIST_HU.md` – 1.0 kiadási ellenőrzőlista
- `docs/RELEASE_CHECKLIST_EN.md` – 1.0 release checklist
- `docs/LANGUAGE_HU.md` – nyelv
- `docs/CLIR_0_1_HU.md` – architektúrafüggetlen assembly / IR
- `docs/CODEGEN_HU.md` – egyszerű kódgenerálási modell
- `docs/LEARNING_HU.md` – tanulási útvonal
- `docs/DESIGN_RULES_HU.md` – egyszerűségi szabályok
- `docs/ONE_ZERO_SCOPE_HU.md` – 1.0 határ
- `docs/STDLIB_HU.md` – kis C-Lite könyvtár

Angol megfelelőik ugyanebben a könyvtárban találhatók.

## 9-target integrációs teszt

A workspace buildje után a teljes külső compiler/assembler határ ellenőrizhető:

```sh
svm_clite/scripts/test_9_targets.sh
```

Ez a külön `svm-clite` és `svm-asm` programot használja, és ugyanazokat a kis C-Lite programokat mind a kilenc targetre lefordítja és assemblálja.
