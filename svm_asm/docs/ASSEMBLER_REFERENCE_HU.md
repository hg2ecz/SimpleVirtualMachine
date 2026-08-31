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

A keresés először az includoló fájl könyvtárához relatív, utána a `-I` könyvtárakban, végül automatikusan az aktuális target beépített `svm_asm/lib/<arch>/` könyvtárában történik. Emiatt például `.include "console.asm"` külön `-I` nélkül is használható. Van rekurzív include, canonical include-once, ciklusdetektálás és 64 szintes maximális mélység. Részletesen: [`SOURCE_INCLUDES_HU.md`](SOURCE_INCLUDES_HU.md).


## Szimbolikus konstansok (`.equ`)

A targetfüggetlen előfeldolgozó egyszerű, kiszámítható konstanshelyettesítést támogat:

```asm
.equ CONSOLE_DATA, 0xFF20
.equ TEN 10
MOVI R1, CONSOLE_DATA
```

A név kis- és nagybetűtől független. A jobb oldal szándékosan egyetlen token: szám, label vagy másik `.equ` neve lehet. A konstansok előre is hivatkozhatnak egymásra; ciklus és többszörös definíció hiba. A funkció nem makrónyelv és nem általános aritmetikai kifejezéskiértékelő.

## Programcímek

Az architektúrák támogatják a program load/entry címének assembler-oldali megadását a saját kézikönyvük szerint. A fizikai CPU RAM `0x0000..0xFEFF`; `0xFF00..0xFFFF` MMIO. A runtime stackkonvenció a felső RAM-ot használja, ezért kézi programnál a választott kód/adat/stack elrendezést a programozónak kell összehangolnia.

## ISA dokumentáció

Minden target saját assembly programming manual + instruction reference párral rendelkezik a megfelelő alkönyvtárban. A normatív közös platform- és ISA-dokumentáció a repository `docs/` könyvtárában van.

## Include-olható könyvtárak

Architektúránként a `svm_asm/lib/<arch>/` alatt:

- `platform.asm` - közös MMIO/platform konstansnevek (`CONSOLE_DATA`, `TEXT_X`, `PALETTE0`, stb.);
- `console.asm` - `putc`, `newline`, `puts`;
- `graphics.asm` - 2 bpp grafika; minden target saját `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, `vline`, `line`, `rect`, `fillrect`, `circle`, `fillcircle` implementációval; a magasabb szintű alakzatok közös `0x00B0..0x00FA` paraméter/scratch blokkot használnak;
- `textscreen.asm` - 40x25 karakteres framebuffer-réteg alapműveletei;
- `math.asm`, `format.asm` - kezdeti kézi ASM matematikai és decimális kiíró könyvtár. Jelenleg teljes referencia-implementáció a Register és Stack targethez van; a többi ISA fokozatosan erre az ABI-szintű mintára egészíthető ki.

Lásd: [`CONSOLE_LIBRARY_HU.md`](CONSOLE_LIBRARY_HU.md), [`GRAPHICS_LIBRARY_HU.md`](GRAPHICS_LIBRARY_HU.md), [`TEXT_SCREEN_LIBRARY_HU.md`](TEXT_SCREEN_LIBRARY_HU.md).

## Eljárások és a nem használt kód elhagyása

A standard assembler-forrásban az eljárásokat explicit módon kell határolni:

```asm
.proc putu16
    ; az eljárás törzse
    RET
.endproc
```

A `.proc NAME` külső, hivatkozható eljárásszimbólumot hoz létre. A cél-assemblerhez
a feldolgozás után a megfelelő `NAME:` címke kerül továbbításra. Az eljáráson belüli
címkék továbbra is közönséges helyi vezérlési címkék lehetnek.

A fordítás az include-ok és a `.equ` konstansok kifejtése után elérhetőségi vizsgálatot
végez. Gyökérnek számít:

- a `.entry NAME` által megnevezett eljárás;
- a `.keep NAME` direktívával explicit megtartott eljárás;
- bármely, eljáráson kívüli forrásrészből szimbolikusan hivatkozott eljárás.

Egy élő eljárás törzsében szereplő másik eljárás neve élővé teszi a hivatkozott
eljárást is. Ez nem csak `CALL` utasításra érvényes: a cím operandusként történő
használata is hivatkozás, ezért függvénypointer- vagy ugrótábla célja sem kerül ki
véletlenül.

Példa:

```asm
.entry start
.include "format.asm"

.proc start
    MOVI R0, 1234
    CALL putu16
    HALT
.endproc
```

Ebben az esetben a `putu16` és az általa használt `putc` bekerül a programba, a
`puti16`, `newline`, `puts` és a többi nem hivatkozott könyvtári eljárás nem.

Megszakításkezelő vagy más, hardver által címzett belépési pont megtartására:

```asm
.keep irq_handler

.proc irq_handler
    ; ...
    IRET
.endproc
```

A `.keep` ismeretlen eljárásneve fordítási hiba, így egy elírás nem okozhat néma
kódelhagyást.

Az optimalizáló nem rendezi át a forrást: a megtartott globális részek és eljárások
az eredeti sorrendjükben maradnak, csak az el nem érhető `.proc` blokkok tűnnek el.
