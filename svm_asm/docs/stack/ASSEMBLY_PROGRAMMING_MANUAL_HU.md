# SVM-S Stack Machine – Assembly programozási kézikönyv


> A jelenlegi videómodellben a normál rendszerterület és a videómemória két külön 16 bites címtér. A normál memóriautasítás nem lép át a videótérbe. A hiteles kiosztás: `../../../docs/PLATFORM_HU.md`.

Ez a dokumentum a jelenlegi költségoptimalizált **SVM-S v2 / executable v3** veremgép programozási kézikönyve. Az assembler Forth-szerű, de a rendszer nem interaktív Forth: a forrás előre gépi kódra fordul.

## Eljárásblokkok és nem használt kód eltávolítása

A nyilvános/hívható rutinokat `.proc NAME` ... `.endproc` blokkokban célszerű írni. Az `.entry NAME` a program belépési eljárását elérhetőségi gyökérré teszi; a `.keep NAME` hardveres callback vagy önálló könyvtári töredék explicit megtartására szolgál. Az assembler az `.include` és `.equ` kifejtése után eltávolítja azokat a `.proc` blokkokat, amelyek ezekből a gyökerekből vagy az élő kódban szereplő szimbolikus hivatkozásokból nem érhetők el. Az eljáráson belüli közönséges címkék továbbra is helyi vezérlési címkék, nem külön elhagyható eljárások.

Stack ISA esetén a `.proc` **nem** generál automatikus `RET` utasítást; visszatérő rutin végére `RET`-et kell írni. A natív `: név ... ;` alak alacsony szintű Stack-assembler konstrukcióként megmarad, de nem procedure-GC határ, ezért újrafelhasználható könyvtári rutinokhoz nem javasolt.


## 1. Programozói modell

A gép:

- 16 bites cellákkal dolgozik;
- 16 bites címtérrel rendelkezik;
- little-endian;
- 64 KiB memóriát lát;
- nincs általános célú regiszterkészlete;
- külön data stacket és közös return/control stacket használ;
- a data stack két legfelső eleme TOS/NOS regiszterpárban cache-elhető; a NOS lusta (lazy) visszatöltésű.

A programozás alapja a stack effect tudatos követése.

Például:

```text
ADD   ( a b -- a+b )
```

A jobb szélső elem a verem teteje.

## 2. Minimális program

```forth
.load 0x0200
.entry main
.proc main
    1 2 + DROP
    HALT
.endproc
```

A régi/alacsony szintű `: név ... ;` forma továbbra is új definíciót hoz létre és a `;` `RET`-et generál, de új kódhoz a `.proc/.endproc` forma javasolt, mert csak ez képez procedure-GC határt. `.proc` esetén a `RET`-et explicit kell kiírni.

## 3. Lexikai szabályok

- whitespace-tokenizált forrás;
- a nevek kis- és nagybetűtől függetlenek;
- `\` karaktertől sor végéig megjegyzés;
- decimális és `0x` hexadecimális számok;
- számokban `_` elválasztó használható;
- ismeretlen szó hívásként fordul, és definícióra/labelre kell feloldódnia.

## 4. Literálok és kódsűrűség

A bare szám automatikusan a legrövidebb formát kapja.

- `-1` / `TRUE`, valamint `0..10`: 1 bájt
- `0..255` nagyobb kis pozitív értékei: `PUSH8`, 2 bájt
- `-128..-2`: `PUSHS8`, 2 bájt
- minden más 16 bites érték: `PUSH16`, 3 bájt

Példa:

```forth
0 1 2 14 255 0xFF06
```

Az assembler választja a megfelelő fizikai kódolást.

`FALSE` = `0`, `TRUE` = `0xFFFF`.

## 5. A data stack

Alapműveletek:

```text
DUP      ( a -- a a )
DROP     ( a -- )
SWAP     ( a b -- b a )
OVER     ( a b -- a b a )
ROT      ( a b c -- b c a )
NIP      ( a b -- b )
TUCK     ( a b -- b a b )
2DUP     ( a b -- a b a b )
2DROP    ( a b -- )
PICK n
ROLL n

> **Assembly-orientált utasítások:** a `NIP`, `TUCK`, `2DUP`, `2DROP`, `PICK` és `ROLL` azért maradnak valódi ISA-utasítások, mert kézzel írt stack/Forth-szerű assemblyben jelentősen javítják a kódsűrűséget és az olvashatóságot. A C backend működéséhez nem szükségesek. Ez tudatos ár–érték döntés, nem compiler-követelmény.
```

A stack programozás legfontosabb szabálya: egy szó stack effectje legyen egyértelmű és lehetőleg dokumentált.

```forth
: square   \ ( n -- n2 )
    DUP *
;
```

## 6. A TOS+NOS lazy stack-cache jelentése

A logikai stack legfelső eleme a CPU belső `TOS`, a második eleme pedig – amikor ténylegesen szükséges – a belső `NOS` regiszterben van. A két regiszter programból nem címezhető. A `NOS` **lusta visszatöltésű**: bináris művelet után a CPU nem olvassa be automatikusan a következő RAM-stack elemet, csak akkor, ha egy későbbi utasítás valóban második operandust kér.

Ennek következményei:

- unáris ALU-művelethez nincs data-stack RAM-hozzáférés;
- ha `TOS` és `NOS` egyaránt érvényes, a bináris ALU-művelet is teljesen regiszteres;
- `SWAP` cache-hit esetén tisztán regisztercsere;
- `NIP` cache-hit esetén csak cache-állapot módosítás;
- `DUP` az első két cache-hely feltöltéséig nem spill-el RAM-ba;
- egy harmadik, majd további push szükség szerint egy 16 bites cellát spill-el a data-stack RAM-ba;
- bináris eredmény `TOS`-ban marad, a következő `NOS` pedig csak igény esetén töltődik vissza.

Ez mikroarchitekturális optimalizáció: az ISA és a stack effectek nem változnak. A ciklusszámláló csak a ténylegesen végrehajtott RAM-hozzáféréseket számolja.

## 7. Aritmetika

```text
+    ADD     ( a b -- a+b )
-    SUB     ( a b -- a-b )
*    MUL     ( a b -- low16(a*b) )
/    DIV     unsigned
MOD          unsigned remainder
NEG NEGATE   ( a -- -a )
1+  INC
1-  DEC
```

A műveletek 16 bitesek és körbefordulnak.

Nullával osztás futási hiba.

## 8. Logikai és shift műveletek

```text
AND OR XOR NOT
SHL SHR
SHL1 / 2*
SHR1 / 2/
```

A bináris `SHL/SHR` a felső stack-elemet shift countként használja, `count & 15` módon.

Egyetlen bites shiftnél a `2*`/`2/` forma a legkisebb kód.

## 9. Összehasonlítás és boolean

```text
=  <>  U<  U>  <  >  0=  0<
```

Explicit aliasok:

```text
EQ NE ULT UGT SLT SGT
```

Az eredmény mindig kanonikus Forth boolean:

```text
false = 0x0000
true  = 0xFFFF
```

## 10. Egyszerű memóriaelérés

```text
C@      ( addr -- value )
@       ( addr -- value )
C!      ( value addr -- )
!       ( value addr -- )
```

Aliasok:

```text
LOAD8 LOAD16 STORE8 STORE16
```

`C@` nullával bővíti a byte-ot. `C!` a value alsó 8 bitjét írja.

A 16 bites `@` és `!` little-endian. `0xFFFF` címen 16 bites elérés nem érvényes.

## 11. Automatikus abszolút memóriaoptimalizálás

Ha a memóriautasítást közvetlenül konstans cím előzi meg, az assembler rövidebb abszolút formát használhat.

Forrás:

```forth
65 0xFF06 C!
```

A programozónak nem kell külön `STORE8ABS` mnemonikot választania. A cím nem kerül feleslegesen a data stackre.

Ezért MMIO-nál természetesen írható a jól olvasható:

```forth
15 0xFF04 C!
```

## 12. Post-increment lineáris memória-primitívek

A költségoptimalizált ISA négy egybájtos lineáris memóriajáró műveletet tartalmaz:

```text
C@+   ( addr -- addr+1 value )
C!+   ( value addr -- addr+1 )
@+    ( addr -- addr+2 value )
!+    ( value addr -- addr+2 )
```

Aliasok:

```text
LOAD8+ STORE8+ LOAD16+ STORE16+
```

Ezek akkor jók, amikor a pointert a feldolgozás után továbbra is meg kell tartani.

### Példa: két pointeres byte másolás

```forth
.load 0x0200
.entry main
.proc main
    0x3000 0x4000
    256 0 DO
        SWAP C@+ ROT C!+
    LOOP
    2DROP
    HALT
.endproc
```

A ciklus után a frissített forrás- és célpointer marad a stacken, majd `2DROP` eltávolítja őket.

## 13. Feltételes szerkezetek

### IF / ELSE / THEN

```forth
condition IF
    ...
ELSE
    ...
THEN
```

Az `IF` elfogyasztja a feltételt.

Példa:

```forth
: abs16   \ ( n -- |n| )
    DUP 0< IF NEG THEN
;
```

## 14. BEGIN ciklusok

Végtelen ciklus:

```forth
BEGIN
    ...
AGAIN
```

Feltételes kilépés:

```forth
BEGIN
    ... condition
UNTIL
```

`UNTIL` akkor lép ki, ha a flag igaz.

WHILE forma:

```forth
BEGIN
    ... condition
WHILE
    ...
REPEAT
```

## 15. DO / LOOP

> A `DO/?DO/I/J/LOOP/+LOOP/LEAVE/UNLOOP` család elsősorban **kézi assembly/Forth-szerű programozhatóság miatt** része az ISA-nak. A C fordító hagyományos branch-alapú ciklusokat is tud generálni, ezért ez a blokk nem compiler-követelmény; a megtartás oka a veremgép természetes programozási modellje és a tömör kézi kód.

A paramétersorrend Forth-szerű:

```text
( limit start -- )
```

Példa:

```forth
10 0 DO
    I DROP
LOOP
```

`I` az aktuális indexet, `J` a következő külső loop indexét teszi a data stackre.

Mivel a ciklus-frame-ek a visszatérési címekkel közös return/control stacken vannak, az `I` és `J` használata annak a wordnek a környezetére értendő, amely az aktív `DO...LOOP` ciklust létrehozta. Egy meghívott word nem feltételezheti, hogy a hívó ciklus-frame-je közvetlenül elérhető `I/J`-vel. Ez tudatos költségoptimalizálási korlátozás, amellyel elkerülhető a harmadik loop stack vagy külön loop-frame pointer.

### ?DO

```forth
10 10 ?DO
    ...
LOOP
```

Ha `start == limit`, a törzs egyszer sem fut le.

### +LOOP

```forth
10 0 DO
    I DROP
    2
+LOOP
```

A step a data stackről fogy el. Pozitív és negatív lépés is támogatott.

### LEAVE

Az aktuális számlált ciklusból azonnal kilép.

### UNLOOP

A legbelső loop frame-et eltávolítja a közös return/control stackről. Aktív `DO` ciklusból kiadott `EXIT` esetén a strukturált assembler automatikusan beszúrja a szükséges `UNLOOP` műveleteket a `RET` elé; kézi alacsony szintű vezérlésnél ugyanezt a szabályt kell betartani.

## 16. CASE

```forth
value CASE
    1 OF
        ...
    ENDOF
    2 OF
        ...
    ENDOF
    ... default ...
ENDCASE
```

## 17. Definíciók és hívások

```forth
: add-one   \ ( n -- n+1 )
    1+
;

: main
    41 add-one DROP
    HALT
;
```

A `;` `RET`-et generál. `EXIT` szintén visszatér. `RECURSE` az aktuális definíciót hívja.

A CPU külön return stacket használ, ezért a data stack adatai és a visszatérési címek nem keverednek.

## 18. Branch relaxation

A strukturált assembler először 8 bites relatív brancheket próbál használni. Ha a cél nem fér `-128..127` távolságba, automatikusan 16 bites abszolút változatra vált.

A programozónak ezért általában nem kell rövid/hosszú branch formát választania.

Ez fontos része a kódméret-optimalizálásnak.

## 19. Memóriatérkép

| Tartomány / cím | Funkció |
|---|---|
| `0x0000..0xFAFF` | program/adat RAM |
| `0xFB00..0xFCFF` | adatverem |
| `0xFD00..0xFEFF` | return/control stack |
| `0xFF00..0xFF01` | billentyűzet |
| `0xFF02..0xFF06` | karakter X/Y, FG/BG és `TEXT_CHAR` |
| `0xFF0B` | VSYNC számláló |
| `0xFF0C..0xFF0F` | négy 4 bites választó a fix 16 színű master palettába |
| külön videótér `0x0000..0x3E7F` | 16 000 bájtos framebuffer |
| külön videótér `0x3E80..0x3FFF` | 384 bájt tartalék |

## 20. Videó: 320x200x2 bpp, egyetlen VRAM

Egyetlen 16 KiB videó-RAM van, bank- és swaplogika nélkül. A videó load/store primitívek csak ezt az adat-videóteret érik el. A 2 bites pixelek négy színhelyet választanak, amelyeket a `0xFF0C..0xFF0F` regiszterek kötnek a fix 16 színű master palettához.

## 21. Belső karakter-ROM és karaktergenerátor

A szövegrács 40x25. A font a videóeszköz belső karakter-ROM-ja, nem CPU-címezhető memória. A karaktert közvetlenül a `TEXT_CHAR` MMIO-val rajzoljuk ki; a kurzort a `TEXT_X/TEXT_Y` regiszterekkel állítjuk.

```forth
5  0xFF02 C!
4  0xFF03 C!
3  0xFF04 C!      \ előtér színhely
0  0xFF05 C!      \ háttér színhely
65 0xFF06 C!
```

A karakterkiíráshoz nem kell firmware-szolgáltatás. A karakter byte-ját a `0xFF06` címre kell írni; a kurzor home művelete `0 0xFF02 C! 0 0xFF03 C!`.

## 22. Billentyűzet polling

```forth
: wait-key   \ ( -- key )
    BEGIN
        0xFF00 C@ 0=
    UNTIL
    0xFF01 C@
;
```

Alacsony szintű programnál ügyelj arra, hogy a közös return/control stack ciklus-frame-jei minden úton helyesen legyenek eltávolítva.

## 23. Host karakterkimenet

A stack implementáció külön host-karakterkimeneti MMIO-t is tartalmaz:

```text
0xFF20 CONSOLE_DATA
0xFF21 CONSOLE_STATUS
```

Példa:

```forth
72 0xFF20 C!      \ H
73 0xFF20 C!      \ I
10 0xFF20 C!      \ newline
```

Ez nem azonos a `TEXT_CHAR` (`0xFF06`) karaktergenerátorral. A `TEXT_CHAR` a framebufferbe rajzol, az `CONSOLE_DATA` a referencia-runtime VT100/RS232 terminálkimenetére ír.

## 24. Költségoptimalizált stack-programozási szabályok

1. Tartsd a stack effecteket röviden és egyértelműen.
2. Használd a kis literálokat természetesen; az assembler a legrövidebb kódot választja.
3. Konstans MMIO-címnél írd közvetlenül a címet a memóriaoperáció elé, hogy az abszolút optimalizálás működhessen.
4. Lineáris pointerhez `C@+`, `C!+`, `@+`, `!+` használata előnyös.
5. Egyszerű lokális vezérlésnél hagyd az assemblert branch-relaxationt végezni.
6. Ne tegyél felesleges `SWAP/ROT` műveleteket a kódba; a stack elrendezését már a szó megtervezésekor válaszd jól.
7. Gyakran ismétlődő stack-szekvenciát érdemes külön colon-wordbe szervezni, ha a hívás költsége megtérül.
8. Ne használd normál RAM-ként a data stack és a közös return/control stack tartományát.

## 25. Gyakori hibák

- stack underflow egy bináris műveletnél;
- egy szó dokumentált és tényleges stack effectjének eltérése;
- `C!` operandussorrendjének felcserélése: `( value addr -- )`;
- `DO` sorrendjének felcserélése: `( limit start -- )`;
- aktív loop frame mellett hibás kézi `EXIT`/`RET` kezelés;
- `@` használata `0xFFFF` címen;
- `TEXT_CHAR` és `CONSOLE_DATA` összekeverése;
- framebuffer-bank váltás után a write/display szerepek figyelmen kívül hagyása.

## 26. Teljes példa: karakterkiírás az egyetlen framebufferbe

Állítsd be a text MMIO-t, majd írd a karakter byte-ját a `TEXT_CHAR` regiszterbe.

## 27. Kapcsolódó dokumentumok

- `../README.md` – assembler/ISA kiindulópont
- `INSTRUCTION_REFERENCE_EN.md` – hiteles hexadecimális opcode referencia
- `../../../docs/PLATFORM.md` – közös géparchitektúra és költség/érték indoklás


## Kétirányú lineáris memóriabejárás
Előrefelé `C@+ C!+ @+ !+`, hátrafelé `C@- C!- @- !-` használható. A hátrafelé mutató cím egy elemmel a tartomány vége után indul, és a hozzáférés előtt csökken. Az -1 és 0..10 literál egybájtos; 11..14 normál kétbájtos literálként kódolódik.

## Gyors 0. lap

A konstans című `0x20 @`, `0x21 C!` jellegű alakokat az assembler automatikusan kétbájtos zero-page utasítássá rövidíti; külön szintaxis és lapregiszter nem kell.

## Timer / interrupt quick reference

A közös gép 32 bites virtuális órát, egy 16 bites időzítőt, valamint timer/VSYNC/billentyűzet IRQ-forrásokat biztosít a `0xFF12..0xFF1F` tartományban. Az IRQ-vektort és a forrásmaszkot letiltott megszakítások mellett állítsd be, a kezelt biteket az `IRQ_ACK` (`0xFF14`) regiszteren nyugtázd, majd `IRET` utasítással térj vissza. A normatív MMIO-szemantikát a projekt szintű `../../../docs/PLATFORM_HU.md` írja le.


## Utasításkódolás és végrehajtási idő

A hex opcode, utasításhossz és ciklusidő táblázatai: [INSTRUCTION_REFERENCE_HU.md](INSTRUCTION_REFERENCE_HU.md).

## Minimális carry állapot

A Stack CPU egyetlen rejtett `C` bitet tart fenn kizárólag többwordös integer aritmetikához. `ADD` és `SHL1` carry-outot, `SUB` no-borrow értéket, `SHR1` a kieső bit0-t írja bele. `ADC`, `SBC` és `RCR1` ezt az állapotot használja. Az összehasonlítások és feltételes vezérlés továbbra is explicit stackértékkel működnek; nincs általános státuszregiszter.

## Grafikai könyvtár

A `graphics.asm` exportálja a gyors `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, `vline` primitíveket, továbbá a magasabb szintű `line`, `rect`, `fillrect`, `circle`, `fillcircle` eljárásokat. ABI: ( szín -- ), paletta ( p0 p1 p2 p3 -- ), putpixel ( x y -- ), clear ( szín -- ), hline ( x0 x1 y -- ), vline ( x y0 y1 -- ). A nem használt eljárásokat a procedure-GC eltávolítja.

Az öt vagy több logikai paramétert igénylő alakzatok minden ISA-n ugyanazt a 16 bites grafikai paraméterblokkot használják: `GFX_X0=0x00C0`, `GFX_Y0=0x00C2`, `GFX_X1=0x00C4`, `GFX_Y1=0x00C6`, `GFX_W=0x00C8`, `GFX_H=0x00CA`, `GFX_R=0x00CC`, `GFX_COLOR=0x00CE`. A `0x00B0..0x00BE` tartomány egyes targeteken belső virtuálisregiszter-scratch; a `0x00D0..0x00FA` tartomány további grafikai scratch/current-color terület. A `graphics.asm` használatakor ezért a teljes `0x00B0..0x00FA` tartomány fenntartott. `line` az `(x0,y0,x1,y1,color)`, `rect/fillrect` az `(x,y,w,h,color)`, `circle/fillcircle` a `(cx,cy,r,color)` mezőket olvassa. A procedure-GC a nem használt alakzatrutinokat és függőségeiket eltávolítja.

