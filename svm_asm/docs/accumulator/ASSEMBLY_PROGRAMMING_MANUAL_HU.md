# Akkumulátoros assembly programozási kézikönyv


> A jelenlegi videómodellben a normál rendszerterület és a videómemória két külön 16 bites címtér. A normál memóriautasítás nem lép át a videótérbe. A hiteles kiosztás: `../../../docs/PLATFORM_HU.md`.

## Eljárásblokkok és nem használt kód eltávolítása

A nyilvános/hívható rutinokat `.proc NAME` ... `.endproc` blokkokban célszerű írni. Az `.entry NAME` a program belépési eljárását elérhetőségi gyökérré teszi; a `.keep NAME` hardveres callback vagy önálló könyvtári töredék explicit megtartására szolgál. Az assembler az `.include` és `.equ` kifejtése után eltávolítja azokat a `.proc` blokkokat, amelyek ezekből a gyökerekből vagy az élő kódban szereplő szimbolikus hivatkozásokból nem érhetők el. Az eljáráson belüli közönséges címkék továbbra is helyi vezérlési címkék, nem külön elhagyható eljárások.


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
.proc start
    LDAI 72
    STA8 0xFF20
    HALT
.endproc
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

## Grafikai könyvtár

A `graphics.asm` exportálja a gyors `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, `vline` primitíveket, továbbá a magasabb szintű `line`, `rect`, `fillrect`, `circle`, `fillcircle` eljárásokat. ABI: A=szín; paletta: X -> 4 bájtos tábla; putpixel: X=x, Y=y; clear: A=szín; hline: X=x0, Y=x1, A=y; vline: A=x, X=y0, Y=y1. A nem használt eljárásokat a procedure-GC eltávolítja.

Az öt vagy több logikai paramétert igénylő alakzatok minden ISA-n ugyanazt a 16 bites grafikai paraméterblokkot használják: `GFX_X0=0x00C0`, `GFX_Y0=0x00C2`, `GFX_X1=0x00C4`, `GFX_Y1=0x00C6`, `GFX_W=0x00C8`, `GFX_H=0x00CA`, `GFX_R=0x00CC`, `GFX_COLOR=0x00CE`. A `0x00B0..0x00BE` tartomány egyes targeteken belső virtuálisregiszter-scratch; a `0x00D0..0x00FA` tartomány további grafikai scratch/current-color terület. A `graphics.asm` használatakor ezért a teljes `0x00B0..0x00FA` tartomány fenntartott. `line` az `(x0,y0,x1,y1,color)`, `rect/fillrect` az `(x,y,w,h,color)`, `circle/fillcircle` a `(cx,cy,r,color)` mezőket olvassa. A procedure-GC a nem használt alakzatrutinokat és függőségeiket eltávolítja.

