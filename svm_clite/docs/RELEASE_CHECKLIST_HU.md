# SVM C-Lite 1.0 kiadási ellenőrzőlista

Ez a lista szándékosan rövid. Az 1.0 célja nem új funkció, hanem egy kis, érthető és megbízható architektúrafüggetlen assembly-szerű nyelv.

## 1. Fordító build

A Rust toolchainnel rendelkező gépen:

```sh
cargo build -p svm-clite
cargo test -p svm-clite
cargo build -p svm-asm
cargo test -p svm-asm
```

Release-blocker minden compiler error, test failure és `svm-clite` warning.

## 2. Nyelvi mag

Az 1.0 nyelvi készlet maradjon:

- `bool`, `i8`, `u8`, `i16`, `u16`, `void`;
- skalár változó;
- fix méretű tömb;
- egy pointer-szint;
- `fn`, paraméter, `return`;
- `if / else`;
- `while`, `break`, `continue`;
- egyszerű aritmetikai, bit- és összehasonlító műveletek;
- `load8/load16/store8/store16`;
- `vload8/vload16/vstore8/vstore16`;
- textual `include`;
- `//` és `/* ... */` komment.

Az 1.0 előtt ne kerüljön be új nyelvi feature.

## 3. Egyszerűségi kapu

A `svm-clite` maradjon:

```text
lexer -> parser -> semantic check -> CLIR -> target ASM
```

Nem kerülhet bele:

- optimizer;
- constant folding;
- SSA;
- regiszterallokátor;
- data-flow pass;
- linker;
- beépített assembler;
- SVM-C függőség;
- általános macro-preprocessor.

## 4. Kilenc target

Mind a kilenc targetre forduljon le ugyanaz a C-Lite forrás:

```text
register
stack
accumulator
memreg
loadstore
regmem
memory2memory
belt
tta
```

Legalább ezekkel a programosztályokkal:

1. aritmetika és signed/unsigned összehasonlítás;
2. `while`, `break`, `continue`;
3. tömb és pointer;
4. függvényhívás több paraméterrel;
5. 8 és 16 bites memóriaelérés;
6. volatile/MMIO;
7. `bool`;
8. globális változó és globális tömb.

A cél helyesség és a target természetes gépmodelljének használata, nem agresszív optimalizálás. A `report_codegen.sh` csak regressziójelző: nyilvánvaló generic-emuláció vagy fölösleges memóriaforgalom ne maradjon.

A fájlalapú külső integrációs ellenőrzéshez a workspace build után:

```sh
svm_clite/scripts/test_9_targets.sh
svm_clite/scripts/report_codegen.sh svm_clite/tests/programs/array_pointer.cl
```

A script a külön `svm-clite` és `svm-asm` binárist használja, és 9 programosztályt futtat mind a 9 targeten (81 fordítás+assemblálás).

## 5. Külső assembler határ

Alap fordítás:

```sh
svm-clite --target register program.cl
```

kimenete target assembly legyen.

A bináris külön assemblerrel készüljön:

```sh
svm-asm register program.asm program.svm
```

A `.proc/.endproc`, `.entry`, `.keep`, assembly include és a nem használt procedure-ök kiszűrése az assembler felelőssége maradjon.

## 6. Include

Ellenőrizendő:

- relatív include;
- `-I` keresési út;
- include-once;
- ciklikus include hiba;
- hiányzó fájlnál fájl és sorszám.

Az include maradjon egyszerű textual beillesztés, macro-rendszer nélkül.

## 7. Dokumentáció

Az alábbiaknak a tényleges implementációt kell leírniuk:

- `PROGRAMMING_MANUAL_HU.md` / `EN`;
- `LANGUAGE_HU.md` / `EN`;
- `CLIR_0_1_HU.md` / `EN`;
- `CODEGEN_HU.md` / `EN`;
- `DESIGN_RULES_HU.md` / `EN`;
- `ONE_ZERO_SCOPE_HU.md` / `EN`.

Release-blocker, ha a dokumentáció olyan nyelvi elemet ígér, amelyet a parser vagy backend nem támogat.

## 8. 1.0 döntés

Ha a build warningmentes, minden teszt zöld, a kilenc target smoke teszt átmegy, és nincs dokumentációs eltérés, az rc kiadható `1.0.0` néven.

Az 1.0 után is az alapelv marad:

> A C-Lite strukturált, architektúrafüggetlen assembly. A nyelv és a fordító egyszerűsége fontosabb a kényelmi feature-öknél és az optimalizált kódnál.
