# Akkumulátoros assembly programozási kézikönyv


> A jelenlegi videómodellben a normál rendszerterület és a videómemória két külön 16 bites címtér. A normál memóriautasítás nem lép át a videótérbe. A hiteles kiosztás: `../../../docs/PLATFORM_HU.md`.

## Programozási modell

A CPU szándékosan kevés állapotot tartalmaz:

- `A`: 16 bites akkumulátor, az aritmetika és a visszatérési érték helye;
- `X`: 16 bites index-/címregiszter és második ALU operandus;
- `Y`: olcsó második cím-/célpointer; nem általános ALU-regiszter;
- `SP`: hardveres veremmutató;
- `PC`: programszámláló;
- `Z/N/C` flag-ek.

A bináris aritmetika alapformája `A op X`, így általános regiszterfájl nélkül is olcsó a kifejezések kiértékelése.

## Egyszerű program

```asm
.load 0
.entry start

start:
    LDAI 72
    STA8 0xFF20
    HALT
```

## Kifejezések

Tipikus minta két részkifejezéshez:

```asm
; bal oldal A-ban
    PUSHA
; jobb oldal kiszámítása A-ba
    TAX
    POPA
    ADDX
```

## Memória

Fix változóhoz és MMIO-hoz a 3 bájtos abszolút alak a legjobb:

```asm
    LDA16 0x6000
    INC
    STA16 0x6000
```

Dinamikus címhez az `X` használható:

```asm
    LDXI 0x8000
    LDAI 0x00FF
    STA8 [X]
```

Lineáris előrefelé bejáráshoz az egybájtos post-increment alakok használhatók:

```asm
copy:
    LDA8 [X+]
```

Hátrafelé másoláshoz a később hozzáadott pre-decrement `[-X]` / `[-Y]` formák használhatók. Általános komplex indexelt címzés továbbra sincs; csak a nagy értékű lineáris előre/hátra bejárás kapott hardvertámogatást.

## Hívás

A `CALL/RET` a hardveres vermet használja. A visszatérési érték `A`-ban van. Az SVM-C statikus paraméter- és lokális tárhelye miatt frame pointer nem szükséges.

## Közös kisgép

A futtatókörnyezet ugyanazt a 64 KiB-os gépprofilt használja, mint a többi SVM CPU: 320x200x2 bpp videó egyetlen külön 16 KiB-os videó-adatcímtérrel, 4-a-16-ból palettával, a videóeszköz belső, CPU-ból nem címezhető karakter-ROM-jával és `0xFF00` kezdetű MMIO-val.


## X/Y memóriamásolási modell
`X` a forrás/index pointer, `Y` pedig az olcsó célcím-regiszter. `Y` szándékosan nem általános ALU-regiszter. Előrefelé `LDA8 [X+]` / `STA8 [Y+]`, hátrafelé `LDA8 [-X]` / `STA8 [-Y]` használható; a 16 bites alakok kettővel lépnek.


## Automatikus rövid branch-ek

Az assembler a lokális `JMP/CALL` és feltételes branch-eket automatikusan 2 bájtos signed PC-relatív formára kódolja, ha a cél elérhető; különben megtartja a 3 bájtos abszolút formát. Külön short forrás-mnemonik nem kell. Az aktuális accumulator executable formátum `SVA\x06`.

## Gyors 0. lap

A `LDA8Z/LDA16Z/STA8Z/STA16Z` kétbájtos formák a `0x00..0xFF` tartományt érik el. Az SVM-C ezeket automatikusan használja a fast-page változókhoz.

## Timer / interrupt gyors referencia

A közös platform 32 bites virtuális órát, egy 16 bites timert és timer/VSYNC/billentyűzet IRQ-forrásokat biztosít a `0xFF12..0xFF1F` tartományban. A vektort és forrásmaszkot tiltott interrupt mellett célszerű beállítani; a kezelt forrásokat az `IRQ_ACK` (`0xFF14`) regiszteren kell nyugtázni, majd `IRET`-tel visszatérni. A normatív MMIO-szemantika: `../../../docs/PLATFORM_HU.md`.


## Utasításkódolás és végrehajtási idő

A hex opcode, utasításhossz és ciklusidő táblázatai: [INSTRUCTION_REFERENCE_HU.md](INSTRUCTION_REFERENCE_HU.md).
