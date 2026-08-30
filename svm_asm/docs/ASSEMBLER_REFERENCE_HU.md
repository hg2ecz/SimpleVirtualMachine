# svm-asm parancssori és forrásreferencia

## Parancssor

```text
svm-asm [-I dir|-Idir] <target> input [output]
```

Targetek: `register`, `stack`, `accumulator`, `memreg`, `loadstore`, `regmem`, `memory2memory`, `belt`, `tta`.

Tipikus kimenetek: `.svm`, `.svs`, `.sva`, `.svf`, `.svl`, `.svr`, `.svc`, `.svb`, `.svt`.

## Include

```asm
.include "console.asm"
```

A keresés először az includoló fájl könyvtárához relatív, utána a `-I` könyvtárakban történik. Van rekurzív include, canonical include-once, ciklusdetektálás és 64 szintes maximális mélység. Részletesen: [`SOURCE_INCLUDES_HU.md`](SOURCE_INCLUDES_HU.md).

## Programcímek

Az architektúrák támogatják a program load/entry címének assembler-oldali megadását a saját kézikönyvük szerint. A fizikai CPU RAM `0x0000..0xFEFF`; `0xFF00..0xFFFF` MMIO. A runtime stackkonvenció a felső RAM-ot használja, ezért kézi programnál a választott kód/adat/stack elrendezést a programozónak kell összehangolnia.

## ISA dokumentáció

Minden target saját assembly programming manual + instruction reference párral rendelkezik a megfelelő alkönyvtárban. A normatív közös platform- és ISA-dokumentáció a repository `docs/` könyvtárában van.

## Include-olható könyvtárak

Architektúránként a `svm_asm/lib/<arch>/` alatt:

- `console.asm` - `putc`, `newline`, `puts`;
- `graphics.asm` - 2 bpp grafikai alapok; a teljes kézi referencia a Register változat;
- `textscreen.asm` - 40x25 karakteres framebuffer-réteg alapműveletei.

Lásd: [`CONSOLE_LIBRARY_HU.md`](CONSOLE_LIBRARY_HU.md), [`GRAPHICS_LIBRARY_HU.md`](GRAPHICS_LIBRARY_HU.md), [`TEXT_SCREEN_LIBRARY_HU.md`](TEXT_SCREEN_LIBRARY_HU.md).
