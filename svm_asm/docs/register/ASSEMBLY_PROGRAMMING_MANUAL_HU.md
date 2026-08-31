# SVM Register CPU – Assembly programozási kézikönyv


> A jelenlegi videómodellben a normál rendszerterület és a videómemória két külön 16 bites címtér. A normál memóriautasítás nem lép át a videótérbe. A hiteles kiosztás: `../../../docs/PLATFORM_HU.md`.

Ez a dokumentum a jelenlegi, költségoptimalizált **SVM Register CPU ISA v2 / executable v5** programozási kézikönyve. A cél nem pusztán az opcode-ok felsorolása, hanem annak bemutatása, hogyan érdemes a gépet hatékonyan programozni.

## Eljárásblokkok és nem használt kód eltávolítása

A nyilvános/hívható rutinokat `.proc NAME` ... `.endproc` blokkokban célszerű írni. Az `.entry NAME` a program belépési eljárását elérhetőségi gyökérré teszi; a `.keep NAME` hardveres callback vagy önálló könyvtári töredék explicit megtartására szolgál. Az assembler az `.include` és `.equ` kifejtése után eltávolítja azokat a `.proc` blokkokat, amelyek ezekből a gyökerekből vagy az élő kódban szereplő szimbolikus hivatkozásokból nem érhetők el. Az eljáráson belüli közönséges címkék továbbra is helyi vezérlési címkék, nem külön elhagyható eljárások.


## 1. A gép programozói modellje

A CPU 16 bites, little-endian rendszer, 64 KiB címtérrel.

- 8 általános célú 16 bites regiszter: `R0..R7`
- 16 bites `PC` programszámláló
- belső `SP` veremmutató
- három állapotjelző: `Z`, `N`, `C`
- minden cím 16 bites
- a 16 bites memóriaértékek little-endian sorrendűek

Az `R0..R7` funkcionálisan egyenértékű. Az `R0..R3` azonban **compact register subset**: több gyakori kétregiszteres utasítás ezekkel 1 bájtos kódolást kap. Emiatt nagy sebességű, belső ciklusokban célszerű a leggyakoribb változókat `R0..R3` között tartani.

## 2. Assembly forrás felépítése

Minimális program:

```asm
.load 0x0200
.entry start
.proc start
    MOVI R0, 1
    HALT
.endproc
```

### Direktívák

- `.load cím` – a program betöltési címe
- `.entry eljárás` – belépési eljárás és procedure-GC gyökér
- `.proc név` / `.endproc` – elhagyható eljárásblokk
- `.keep név` – eljárás explicit megtartása
- `.include "fájl"` – forráskönyvtár behúzása
- `.equ név, érték` – szimbolikus konstans

### Megjegyzés

A `;` karaktertől a sor végéig megjegyzés írható.

### Számok

Elfogadott példák:

```text
1234
0x1234
0xFF06
```

A regiszterek `R0..R7` alakban írhatók. A mnemonikák kis- és nagybetűtől függetlenül használhatók.

## 3. Ajánlott regiszterhasználat

A hardver nem ír elő calling conventiont, de kézi assemblyhez praktikus konvenció:

| Regiszter | Ajánlott szerep |
|---|---|
| R0 | elsődleges pointer / argumentum |
| R1 | második pointer / argumentum |
| R2 | számláló / ideiglenes adat |
| R3 | ideiglenes adat / eredmény |
| R4..R6 | hosszabb életű változók |
| R7 | scratch vagy függvényen belüli ideiglenes |

Ez csak ajánlás. Az `R0..R3` előnyben részesítése csökkenti a kódméretet.

## 4. Compact kódolás

A következő utasítások 1 bájtosak, ha mindkét operandus `R0..R3`:

```text
MOV ADD SUB CMP LOAD8 STORE8 XOR
```

Például:

```asm
ADD R0, R1
```

1 bájt, míg:

```asm
ADD R4, R5
```

2 bájt.

Az assembler automatikusan választja a rövidebb alakot; nincs külön compact mnemonic.

Az egyregiszteres műveletek mind a nyolc regiszterre 1 bájtosak:

```text
NOT NEG INC DEC SHL1 SHR1 PUSH POP
```

Az immediate családok 3 bájtosak:

```text
MOVI ADDI SUBI CMPI
```

## 5. Adatmozgatás

### MOV

```asm
MOV Rd, Rs
```

`Rd = Rs`.

### MOVI

```asm
MOVI Rd, 0x1234
```

16 bites konstanst tölt a regiszterbe.

### CLR pszeudóutasítás

```asm
CLR R2
```

Az assembler ezt meglévő `XOR R2,R2` alakra fordítja. Nem fogyaszt külön opcode-ot; az XOR a normál kétregiszteres családban marad.

## 6. Aritmetika

```text
ADD Rd,Rs
ADDI Rd,imm16
SUB Rd,Rs
SUBI Rd,imm16
MUL Rd,Rs
DIV Rd,Rs
MOD Rd,Rs
NEG Rd
INC Rd
DEC Rd
```

Az aritmetika 16 bites, túlcsorduláskor modulo 65536 szerint körbefordul.

`DIV` és `MOD` előjel nélküli műveletek. Nulla osztó futási hibát eredményez.

Az assembler automatikusan rövidít:

```asm
ADDI R0, 1
```

→ `INC R0`, illetve:

```asm
SUBI R0, 1
```

→ `DEC R0`.

## 7. Logikai és shift műveletek

```text
AND Rd,Rs
OR  Rd,Rs
XOR Rd,Rs
NOT Rd
SHL Rd,Rs
SHR Rd,Rs
SHL1 Rd
SHR1 Rd
```

A változó shiftnél a shift count `Rs & 15`.

A `SHL1` és `SHR1` különösen előnyös pixel-, maszk- és címkezeléshez, mert 1 bájtos.

## 8. Flag-ek

A CPU három flaget tart fenn:

- `Z` – az eredmény nulla
- `N` – az eredmény 15. bitje 1
- `C` – összeadásnál carry, kivonásnál/összehasonlításnál no-borrow

A CPU-ban nincs signed-overflow (`V`) flag.

### Összehasonlítás

```asm
CMP  R0, R1
CMPI R0, 100
```

A regisztereket nem módosítja, a flag-eket úgy állítja be, mintha `R0-R1`, illetve `R0-100` történt volna.

### TEST pszeudóutasítás

```asm
TEST R0
```

Az assembler `OR R0,R0` alakra fordítja. `Z/N` ellenőrzéshez praktikus.

## 9. Feltételes és feltétel nélküli ugrás

```text
JMP  label
CALL label
JZ   label
JNZ  label
JC   label
JNC  label
JN   label
JNN  label
RET
```

Az ugrások 16 bites abszolút cémet tartalmaznak.

Tipikus ciklus:

```asm
    MOVI R0, 100
loop:
    ; ...
    DEC R0
    JNZ loop
```

## 10. CALL, RET és a hardververem

A `CALL` a visszatérési címet a CPU belső memóriaveremére teszi, majd a célra ugrik. A `RET` innen veszi vissza a címet.

A `PUSH Rn` és `POP Rn` ugyanazt a hardververemet használja, ezért függvényekben gondosan párosítani kell őket.

Példa:

```asm
CALL add_one
HALT

add_one:
    INC R0
    RET
```

Regisztermentés:

```asm
worker:
    PUSH R4
    PUSH R5
    ; ...
    POP R5
    POP R4
    RET
```

## 11. Normál indirekt memóriaelérés

```asm
LOAD8  Rd, [Ra]
LOAD16 Rd, [Ra]
STORE8 [Ra], Rs
STORE16 [Ra], Rs
```

- `LOAD8` nullával bővíti a byte-ot 16 bitre.
- `STORE8` csak a forrás alsó 8 bitjét írja.
- `LOAD16/STORE16` little-endian cellával dolgozik.
- 16 bites hozzáférés `0xFFFF` címen nem érvényes.

## 12. Post-increment memóriaelérés

A lineáris memóriajárás költségoptimalizált primitívjei:

```asm
LOAD8  R2, [R0+]     ; R2 = mem8[R0],  R0 += 1
STORE8 [R1+], R2     ; mem8[R1] = R2,  R1 += 1
LOAD16 R2, [R0+]     ; R2 = mem16[R0], R0 += 2
STORE16 [R1+], R2    ; mem16[R1] = R2, R1 += 2
```

A post-increment loadnál a cél- és címregiszternek különböznie kell.

### 256 byte másolása

```asm
.load 0x0200
.entry start
.proc start
    MOVI R0, 0x3000
    MOVI R1, 0x4000
    MOVI R2, 256
copy:
    LOAD8  R3, [R0+]
    STORE8 [R1+], R3
    DEC R2
    JNZ copy
    HALT
.endproc
```

Ez a javasolt forma lineáris buffer-, string- és framebuffer-műveletekhez.

## 13. Memóriatérkép

| Cím | Funkció |
|---|---|
| `0x0000..0xFAFF` | program/adat RAM (a felső 1 KiB runtime-stack konvenció alatt) |
| `0xFF00..0xFF01` | billentyűzet |
| `0xFF02..0xFF06` | karakterpozíció, FG/BG és `TEXT_CHAR` |
| `0xFF0B` | VSYNC számláló |
| `0xFF0C..0xFF0F` | négy 4 bites színválasztó a fix 16 színű master palettába |
| `0xFB00..0xFEFF` | CPU stack |
| külön videótér `0x0000..0x3E7F` | 16 000 bájtos framebuffer |
| külön videótér `0x3E80..0x3FFF` | 384 bájt fenntartott VRAM |

## 14. Videó: 320x200, 2 bpp, egyetlen 16 KiB VRAM

A framebuffer 16 000 bájtos, külön adat-videótérben van. Nincs videobank, dupla puffer vagy swap. A 2 bites pixelérték a négy színhely egyikét választja; a `0xFF0C..0xFF0F` regiszterek megmondják, hogy a négy hely a fix 16 színű master paletta mely színeit jelentse.

## 15. Belső karakter-ROM és karaktergenerátor

A 40x25-ös szövegrács 8x8-as glyph-jei a videóeszköz belső, írásvédett karakter-ROM-jában vannak; ez nem része a CPU címtérének. A karaktert a `TEXT_CHAR` (`0xFF06`) regiszterbe írva rajzoljuk ki. A kurzorpozíció a `TEXT_X/TEXT_Y` regiszterek közvetlen írásával állítható.

## 16. Billentyűzet

`KEY_STATUS` (`0xFF00`) jelzi, hogy van-e olvasható karakter; `KEY_CODE` (`0xFF01`) tartalmazza a kódot.

Egyszerű polling:

```asm
    MOVI R0, 0xFF00
    MOVI R1, 0xFF01
wait_key:
    LOAD8 R2, [R0]
    TEST R2
    JZ wait_key
    LOAD8 R3, [R1]
```

## 17. Kódméret-optimalizálási szabályok

1. A belső ciklusok leggyakoribb operandusait `R0..R3` között tartsd.
2. Egybites shifthez `SHL1/SHR1` használata előnyösebb, mint shift-count regisztert tölteni.
3. Lineáris pointerjáráshoz `[Rn+]` formát használj külön `INC` helyett.
4. `ADDI Rn,1` és `SUBI Rn,1` írható olvashatóbban is; az assembler rövidíti.
5. `CLR` és `TEST` használható kódolási többlet nélkül.
6. MMIO-címeket érdemes hosszabb ideig regiszterben tartani, ha többször használod őket.
7. A szabad opcode-hely nem cél; a programban meglévő rövid primitíveket használd.

## 18. Gyakori hibák

- `LOAD8 R0,[R0+]` – tiltott, mert a betöltött adat és az új pointer ugyanabba a regiszterbe kerülne.
- 16 bites load/store `0xFFFF` címen – érvénytelen.
- A belső karakter-ROM nem CPU-címezhető; karakterrajzoláshoz a `TEXT_CHAR` MMIO-regisztert használd.
- `CALL` után elfelejtett `RET` vagy kiegyensúlyozatlan `PUSH/POP` – hibás visszatéréshez vezethet.
- `C` flaget signed overflowként értelmezni – a CPU-nak nincs `V` flagje.
- framebuffer-műveletnél CPU-memóriautasítást használni a dedikált videómemória-utasítás helyett.

## 19. Teljes példa: karakterkiírás a belső karakter-ROM-mal

Állítsd be a `TEXT_X/TEXT_Y`, `TEXT_FG/TEXT_BG` regisztereket, majd írd a karakter byte-ját a `TEXT_CHAR` (`0xFF06`) regiszterbe.

## 20. Kapcsolódó dokumentumok

- `../README.md` – assembler/ISA kiindulópont
- `INSTRUCTION_REFERENCE_EN.md` – hiteles hexadecimális opcode referencia
- `../../../docs/PLATFORM.md` – közös géparchitektúra és költség/érték indoklás


## Kétirányú lineáris memóriabejárás
Előrefelé `[Rn+]`, hátrafelé `[-Rn]` használható. A pre-decrement byte művelet előtt 1-gyel, word művelet előtt 2-vel csökkenti a címregisztert. Így külön blokk-másoló utasítás nélkül is tömör, átfedésbiztos `memmove` ciklus írható.

## Gyors 0. lap

A `ZLOAD8/ZLOAD16/ZSTORE8/ZSTORE16` kétbájtos formák a `0x00..0xFF` tartományt érik el, implicit R0 használatával. Az SVM-C a `0x00..0xEF` területet gyors statikus adatokra használja, a generált programkód pedig `0x0100` címen indul.

## Timer / interrupt quick reference

A közös gép 32 bites virtuális órát, egy 16 bites időzítőt, valamint timer/VSYNC/billentyűzet IRQ-forrásokat biztosít a `0xFF12..0xFF1F` tartományban. Az IRQ-vektort és a forrásmaszkot letiltott megszakítások mellett állítsd be, a kezelt biteket az `IRQ_ACK` (`0xFF14`) regiszteren nyugtázd, majd `IRET` utasítással térj vissza. A normatív MMIO-szemantikát a projekt szintű `../../../docs/PLATFORM_HU.md` írja le.


## Utasításkódolás és végrehajtási idő

A hex opcode, utasításhossz és ciklusidő táblázatai: [INSTRUCTION_REFERENCE_HU.md](INSTRUCTION_REFERENCE_HU.md).


## Register ISA v3 kódsűrűségi változás

A `B0..BF` egybájtos compact család `AND` műveletet kódol `R0..R3` között; az `XOR` továbbra is teljes értékű általános kétregiszteres utasítás. A `SUBI` hardveres immediate család megmarad, mert az `ADDI -imm16` ugyanazt a numerikus eredményt adja, de a carry/no-borrow flag szemantikája nem minden esetben azonos. A maszkos `AND` gyakorisága miatt a compact hely átcsoportosítása továbbra is jó ár–értékű.

## Grafikai könyvtár

A `graphics.asm` exportálja a gyors `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, `vline` primitíveket, továbbá a magasabb szintű `line`, `rect`, `fillrect`, `circle`, `fillcircle` eljárásokat. ABI: R0=szín; paletta: R0..R3; putpixel: R0=x,R1=y; clear: R0=szín; hline: R0=x0,R1=x1,R2=y; vline: R0=x,R1=y0,R2=y1. A nem használt eljárásokat a procedure-GC eltávolítja.

Az öt vagy több logikai paramétert igénylő alakzatok minden ISA-n ugyanazt a 16 bites grafikai paraméterblokkot használják: `GFX_X0=0x00C0`, `GFX_Y0=0x00C2`, `GFX_X1=0x00C4`, `GFX_Y1=0x00C6`, `GFX_W=0x00C8`, `GFX_H=0x00CA`, `GFX_R=0x00CC`, `GFX_COLOR=0x00CE`. A `0x00B0..0x00BE` tartomány egyes targeteken belső virtuálisregiszter-scratch; a `0x00D0..0x00FA` tartomány további grafikai scratch/current-color terület. A `graphics.asm` használatakor ezért a teljes `0x00B0..0x00FA` tartomány fenntartott. `line` az `(x0,y0,x1,y1,color)`, `rect/fillrect` az `(x,y,w,h,color)`, `circle/fillcircle` a `(cx,cy,r,color)` mezőket olvassa. A procedure-GC a nem használt alakzatrutinokat és függőségeiket eltávolítja.


## Typed arithmetic reference include

The Register standard library also contains `typed_arith.asm` and `typed_convert.asm` as an educational typed arithmetic/conversion reference. See the common `TYPED_ARITHMETIC_LIBRARY_HU.md` / `TYPED_ARITHMETIC_LIBRARY_EN.md` documentation. The portable full IEEE soft-float implementation remains in SVM-C.
