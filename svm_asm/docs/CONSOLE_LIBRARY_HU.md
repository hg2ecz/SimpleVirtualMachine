# Assembly konzolkönyvtár

Az `svm_asm/lib/<arch>/console.asm` fájlok mind a kilenc ISA-hoz include-olható karakteres konzolrutint adnak. A konzol adatregisztere `0xFF20`.

## API

Minden architektúrában ugyanazok a `.proc` eljárások állnak rendelkezésre:

- `putc` – egy karakter kiírása;
- `newline` – CR+LF (`13,10`) kiírása;
- `puts` – RAM-ban lévő, `0` bájttal lezárt sztring kiírása.

Az operandusátadás ISA-függő; ezt minden `console.asm` fejléc-kommentje rögzíti. Az assembler jelenleg nem rendelkezik `.byte`/`.ascii` adatdirektívával, ezért a `puts` nem assembly string-literált, hanem már RAM-ban lévő NUL-lezárt bájtsorozatot kap.

## Include

Példa Register ISA esetén:

```asm
.include "console.asm"
```

Fordításkor:

```sh
svm-asm register program.asm program.svm
```

A kiválasztott target `lib/<arch>/` könyvtárát az assembler automatikusan keresi, ezért a standard könyvtárhoz nem szükséges `-I`. Az architektúrákhoz tartozó könyvtárak: `register`, `stack`, `accumulator`, `memreg`, `loadstore`, `regmem`, `memory2memory`, `belt`, `tta`.

## Kódelhagyás

A `console.asm` rutinjai `.proc/.endproc` blokkok. A teljes fájl nyugodtan include-olható: a végső programba csak az élő kódból elérhető `putc`, `newline`, `puts` és tranzitív függőségeik kerülnek.
