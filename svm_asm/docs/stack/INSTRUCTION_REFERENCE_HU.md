# Veremgépes utasításreferencia

> **Jelölés:** az „assembly-oriented” azt jelenti, hogy az utasítás megtartásának fő indoka a kézzel írt stack/Forth-szerű assembly programozhatóság; a C backend nem igényli. Az utasítás ettől még teljes értékű, stabil ISA-primitív.

Ez a dokumentum a jelenlegi költségoptimalizált veremgépes ISA (`SVS\x08`) normatív programozói utasításdefiníciója. A gép 16 bites cellákat, kétcellás lusta `TOS/NOS` stack-cache-t, adatvermet és közös return/control stacket használ. A többbájtos 16 bites értékek little-endian kódolásúak.

## Kódolás, hossz és végrehajtási idő – gyors referencia

A Stack időzítéséhez egy explicit konvenció szükséges a cache-elt TOS miatt. A ciklusmodell: `../../../svm_rt/docs/CYCLE_MODEL.md`.

- utasításbájt fetch: 1 ciklus/bájt;
- 8 bites adat-hozzáférés: +1 ciklus;
- 16 bites adat/stack-hozzáférés: +2 ciklus;
- TOS és NOS regiszterhozzáférés nem kerül külön VM-ciklusba;
- push csak akkor spill-el egy 16 bites cache-elt cellát, ha TOS és NOS is foglalt: `S=2`;
- bináris művelet csak akkor tölti vissza NOS-t RAM-ból, ha NOS érvénytelen és valóban kell második operandus: `N=2`;
- pop csak akkor tölti vissza az új TOS-t RAM-ból, ha NOS érvénytelen és marad RAM-backed elem: `R=2`;
- bináris művelet után NOS érvénytelen marad, nincs eager refill;
- `MUL/DIV/MOD`: +16 belső ciklus; `MULQ15`: +17; változó `SHL/SHR`: +1.

Ezért néhány idő cache-állapotfüggő. Az alábbi `S`, `N`, `R` csak akkor jelent valós RAM-hozzáférést, amikor az ténylegesen bekövetkezik.

| Assembly forma | Hex | Bájt | Ciklus | Megjegyzés |
|---|---:|---:|---:|---|
| `NOP`, `HALT` | `00`,`01` | 1 | 1 | csak fetch |
| `RET` | `02` | 1 | 3 | return-stack read |
| `DUP` | `03` | 1 | `1+S` = 1 vagy 3 | csak két foglalt cache-cella esetén spill |
| `DROP` | `04` | 1 | `1` vagy `3` | +R, ha marad alatta elem |
| `SWAP` | `05` | 1 | `1+N` = 1 vagy 3 | regisztercsere, ha NOS cache-elt |
| `OVER` | `06` | 1 | 3 | vagy második elem read, vagy egy cache-cella spill |
| `ROT` | `07` | 1 | 5 vagy 7 | harmadik cella read/write; +N, ha NOS lazy |
| `NIP` | `08` | 1 | `1+N` = 1 vagy 3 | NOS hit esetén csak cache-állapotváltozás |
| `TUCK` | `09` | 1 | 3 vagy 5 | egy stack-word spill; +N lazy NOS-nál |
| `2DUP` | `0A` | 1 | 5 | összesen két word-hozzáférés |
| `2DROP` | `0B` | 1 | 1 vagy 3 | legfeljebb egy új-TOS refill |
| `C@+`, `C@-` | `0C`,`3C` | 1 | `2+S` = 2 vagy 4 | byte read; push spill-elhet |
| `@+`, `@-` | `0E`,`3E` | 1 | `3+S` = 3 vagy 5 | word read; push spill-elhet |
| `C!+`, `C!-` | `0D`,`3D` | 1 | `2+N` = 2 vagy 4 | byte write; érték NOS-ból |
| `!+`, `!-` | `0F`,`3F` | 1 | `3+N` = 3 vagy 5 | word write; érték NOS-ból |
| bináris ALU/compare `+ - AND OR XOR = ...` | `10,11,18..1A,20..25` | 1 | `1+N` = 1 vagy 3 | NOS-valid esetben tisztán regiszteres; eredmény után NOS lazy |
| `MUL/DIV/MOD` | `12..14` | 1 | 17 vagy 19 | +16 belső, opcionális lazy NOS refill |
| unáris `NEG,1+,1-,NOT,2*,2/,0=,0<` | `15..17,1B,1E,1F,26,27` | 1 | 1 | csak TOS |
| változó `SHL/SHR` | `1C`,`1D` | 1 | 2 vagy 4 | +1 belső, opcionális NOS refill |
| `C@` | `28` | 1 | 2 | cím már TOS-ban |
| `@` | `29` | 1 | 3 | word adat-read |
| `C!` | `2A` | 1 | `2+N+R` = 2,4,6 | érték NOS-ból, majd opcionális új-TOS refill |
| `!` | `2B` | 1 | `3+N+R` = 3,5,7 | érték NOS-ból, majd opcionális új-TOS refill |
| egybájtos literál `-1,0..10` | `30..3B` | 1 | `1+S` = 1 vagy 3 | spill csak két foglalt cache-cellánál |
| `PUSH8/PUSHS8` | `40/41 ii` | 2 | `2+S` = 2 vagy 4 | literálfetch + opcionális spill |
| `BRA8` | `42 dd` | 2 | 2 | nincs pipeline penalty |
| `BZ8/BNZ8` | `43/44 dd` | 2 | 2 vagy 4 | +R, ha a flag pop után backed elem kerül elő |
| `CALL8` | `45 dd` | 2 | 4 | return-stack write |
| `PICK depth` | `4A dd` | 2 | depth 0: `2+S`; depth>0: `4+S` | kiválasztott backed elem read |
| `ROLL depth` | `4B dd` | 2 | mélységfüggő | több backed read/write |
| zero-page byte load | `4C aa` | 2 | `3+S` | byte read + push |
| zero-page word load | `4D aa` | 2 | `4+S` | word read + push |
| zero-page byte store | `4E aa` | 2 | 3 vagy 5 | +R fogyasztás után, ha marad backed elem |
| zero-page word store | `4F aa` | 2 | 4 vagy 6 | +R fogyasztás után, ha marad backed elem |
| `SYS EI/DI` | `50 00/01` | 2 | 2 | prefix + subopcode |
| `SYS IRET` | `50 02` | 2 | 6 | két return-stack read |
| `SYS ASR1` | `50 03` | 2 | 2 | csak TOS |
| `SYS MULQ15` | `50 04` | 2 | 19 vagy 21 | +17 belső + opcionális NOS refill |
| videó `VC@/VC!` | `50 10/12` | 2 | `C@/C!` analóg +1 prefixbájt | külön VRAM |
| videó `V@/V!` | `50 11/13` | 2 | `@/!` analóg +1 prefixbájt | külön VRAM |
| `PUSH16 imm16` | `80 lo hi` | 3 | `3+S` = 3 vagy 5 | opcionális spill |
| `JMP addr16` | `81 lo hi` | 3 | 3 | abszolút |
| `JZ/JNZ addr16` | `82/83 lo hi` | 3 | 3 vagy 5 | flag pop után lehetséges refill |
| `CALL addr16` | `84 lo hi` | 3 | 5 | return-stack write |
| abszolút byte load | `89 lo hi` | 3 | `4+S` | adat-read + push |
| abszolút word load | `8A lo hi` | 3 | `5+S` | word read + push |
| abszolút byte store | `8B lo hi` | 3 | 4 vagy 6 | lehetséges refill |
| abszolút word store | `8C lo hi` | 3 | 5 vagy 7 | lehetséges refill |

A loop-utasítások a közös return/control stacket használják, ezért a valódi frame-hozzáférési költségük látható:

| Loop utasítás | Rövid/hosszú bájt | Ciklus | Útvonalfüggés |
|---|---:|---:|---|
| `DO` | 1 | 9 vagy 11 | +2, ha a két elfogyasztott paraméter alatt marad adat |
| `I`, `J` | 1 | `5+S` | két return-stack word read, majd index push |
| `UNLOOP` | 1 | 1 | csak pointerfrissítés |
| `?DO8` | 2 | equal: 6 vagy 8; enter: 10 vagy 12 | belépés két loop-frame cellát ír |
| `?DO` | 3 | equal: 7 vagy 9; enter: 11 vagy 13 | mint fent, plusz egy címbájt |
| `LOOP8` | 2 | exit: 6; continue: 8 | continue frissített indexet ír |
| `LOOP` | 3 | exit: 7; continue: 9 | abszolút cél +1 fetchbájt |
| `+LOOP8` | 2 | exit: `6+R`; continue: `8+R` | step az adatstackről pop |
| `+LOOP` | 3 | exit: `7+R`; continue: `9+R` | ugyanaz, abszolút forma |
| `LEAVE8` / `LEAVE` | 2 / 3 | 2 / 3 | loop-frame eltávolítás pointerfrissítés |

A `ROLL n` szándékosan mélységfüggő, mert valódi backed cellákat mozgat, és ezt a költséget nem rejtjük fix névleges idő mögé.

## Stack-jelölés

A stack effect formája:

`( előtte -- utána )`

A jobb szélső elem az adatstack teteje. Boolean igaz = `0xFFFF`, hamis = `0x0000`.

## Utasításhossz-szabály

Az opcode felső két bitje a teljes utasításhosszt kódolja:

- `00xxxxxx` -> 1 bájt
- `01xxxxxx` -> 2 bájt
- `10xxxxxx` -> 3 bájt
- `11xxxxxx` -> 4 bájt (jelenleg nem használt/fenntartott)

Ez olcsó utasításhossz-dekódolást tesz lehetővé.

## Egybájtos mag- és stack-utasítások

| Mnemonik | Hex | Stack effect | Definíció |
|---|---:|---|---|
| `NOP` | `00` | `( -- )` | Nincs művelet. |
| `HALT` | `01` | `( -- )` | CPU leállítása. |
| `RET` / `EXIT` | `02` | `( -- )` | Return address pop a return/control stackről `PC`-be. Strukturált loopból `EXIT` előtt az assembler beilleszti a szükséges `UNLOOP`-okat. |
| `DUP` | `03` | `( a -- a a )` | TOS duplikálása. |
| `DROP` | `04` | `( a -- )` | TOS eldobása. |
| `SWAP` | `05` | `( a b -- b a )` | Felső két cella cseréje. |
| `OVER` | `06` | `( a b -- a b a )` | Második cella másolása TOS-ra. |
| `ROT` | `07` | `( a b c -- b c a )` | Felső három cella forgatása. |
| `NIP` | `08` | `( a b -- b )` | Második cella eltávolítása; **assembly-oriented**. |
| `TUCK` | `09` | `( a b -- b a b )` | TOS másolása a második elem alá; **assembly-oriented**. |
| `2DUP` | `0A` | `( a b -- a b a b )` | Felső pár duplikálása; **assembly-oriented**. |
| `2DROP` | `0B` | `( a b -- )` | Felső pár eldobása; **assembly-oriented**. |

## Egybájtos post-increment memóriabejárók

Ezek költségoptimalizált lineáris memória-primitívek. A memória-hozzáférést és címfrissítést külön `INC`/`2 +` nélkül egyesítik.

| Mnemonik | Alias | Hex | Stack effect | Definíció |
|---|---|---:|---|---|
| `C@+` | `LOAD8+` | `0C` | `( addr -- addr+1 value )` | Unsigned byte read, előreléptetett cím megmarad. |
| `C!+` | `STORE8+` | `0D` | `( value addr -- addr+1 )` | Alsó byte store, előreléptetett cím megmarad. |
| `@+` | `LOAD16+` | `0E` | `( addr -- addr+2 value )` | 16 bites cella read, előreléptetett cím megmarad. |
| `!+` | `STORE16+` | `0F` | `( value addr -- addr+2 )` | 16 bites cella store, előreléptetett cím megmarad. |

## Egybájtos aritmetikai és bitműveletek

| Mnemonik | Hex | Stack effect | Definíció |
|---|---:|---|---|
| `+` / `ADD` | `10` | `( a b -- a+b )` | 16 bites wrapping összeadás. |
| `-` / `SUB` | `11` | `( a b -- a-b )` | 16 bites wrapping kivonás. |
| `*` / `MUL` | `12` | `( a b -- a*b )` | Szorzat alsó 16 bitje. |
| `/` / `DIV` | `13` | `( a b -- a/b )` | Unsigned osztás; 0-val osztás trap. |
| `MOD` | `14` | `( a b -- a%b )` | Unsigned maradék; 0-val osztás trap. |
| `NEGATE` / `NEG` | `15` | `( a -- -a )` | Kétkomplementes negálás. |
| `1+` / `INC` | `16` | `( a -- a+1 )` | Növelés. |
| `1-` / `DEC` | `17` | `( a -- a-1 )` | Csökkentés. |
| `AND` | `18` | `( a b -- a&b )` | Bitenkénti AND. |
| `OR` | `19` | `( a b -- a|b )` | Bitenkénti OR. |
| `XOR` | `1A` | `( a b -- a^b )` | Bitenkénti XOR. |
| `NOT` | `1B` | `( a -- ~a )` | Bitenkénti NOT. |
| `LSHIFT` / `SHL` | `1C` | `( value count -- result )` | Logikai bal shift `count & 15` bittel. |
| `RSHIFT` / `SHR` | `1D` | `( value count -- result )` | Logikai jobb shift `count & 15` bittel. |
| `2*` / `SHL1` | `1E` | `( a -- a<<1 )` | Egybites logikai bal shift. |
| `2/` / `SHR1` | `1F` | `( a -- a>>1 )` | Egybites logikai jobb shift. |

## Egybájtos összehasonlítások

Minden összehasonlítás `0xFFFF` értéket ad igaz, `0x0000` értéket hamis esetben.

| Mnemonik | Hex | Stack effect | Definíció |
|---|---:|---|---|
| `=` / `EQ` | `20` | `( a b -- flag )` | igaz, ha `a == b` |
| `<>` / `NE` | `21` | `( a b -- flag )` | igaz, ha `a != b` |
| `U<` / `ULT` | `22` | `( a b -- flag )` | unsigned `a < b` |
| `U>` / `UGT` | `23` | `( a b -- flag )` | unsigned `a > b` |
| `<` / `SLT` | `24` | `( a b -- flag )` | signed 16 bites `a < b` |
| `>` / `SGT` | `25` | `( a b -- flag )` | signed 16 bites `a > b` |
| `0=` | `26` | `( a -- flag )` | igaz, ha nulla |
| `0<` | `27` | `( a -- flag )` | igaz, ha signed értékként negatív |

## Egybájtos memória- és loop-frame utasítások

| Mnemonik | Hex | Stack effect | Definíció |
|---|---:|---|---|
| `C@` / `LOAD8` | `28` | `( addr -- value )` | Unsigned byte read, cím helyére érték kerül. |
| `@` / `LOAD16` | `29` | `( addr -- value )` | 16 bites cella read. |
| `C!` / `STORE8` | `2A` | `( value addr -- )` | Alsó byte store. |
| `!` / `STORE16` | `2B` | `( value addr -- )` | 16 bites cella store. |
| `DO` | `2C` | `( limit start -- )` | Kétcellás `(limit,index=start)` loop frame push a közös return/control stackre. |
| `I` | `2D` | `( -- index )` | Aktuális loop index push. |
| `J` | `2E` | `( -- outer-index )` | Külső loop indexének push-a. |
| `UNLOOP` | `2F` | `( -- )` | Aktuális loop frame eltávolítása. |

## Sűrű egybájtos literálablak

Ezek immediate bájt nélkül pusholnak literált; az assembler automatikusan választja őket.

| Forrásliterál | Hex | Stack effect |
|---:|---:|---|
| `-1` / `TRUE` | `30` | `( -- FFFF )` |
| `0` | `31` | `( -- 0000 )` |
| `1` | `32` | `( -- 0001 )` |
| `2` | `33` | `( -- 0002 )` |
| `3` | `34` | `( -- 0003 )` |
| `4` | `35` | `( -- 0004 )` |
| `5` | `36` | `( -- 0005 )` |
| `6` | `37` | `( -- 0006 )` |
| `7` | `38` | `( -- 0007 )` |
| `8` | `39` | `( -- 0008 )` |
| `9` | `3A` | `( -- 0009 )` |
| `10` | `3B` | `( -- 000A )` |

## Kétbájtos utasítások

A második bájt immediate, signed relatív offset vagy depth paraméter.

| Mnemonik | Hex | Operandusbájt | Stack effect | Definíció |
|---|---:|---|---|---|
| `PUSH8 u8` | `40` | unsigned 8 bites literál | `( -- value )` | Zero-extend 16 bitre. |
| `PUSHS8 s8` | `41` | signed 8 bites literál | `( -- value )` | Sign-extend 16 bitre. |
| `BRA8 rel8` | `42` | signed PC-relatív offset | `( -- )` | Feltétel nélküli relatív branch. |
| `BZ8 rel8` | `43` | signed PC-relatív offset | `( flag -- )` | Branch, ha a popolt érték nulla. |
| `BNZ8 rel8` | `44` | signed PC-relatív offset | `( flag -- )` | Branch, ha a popolt érték nem nulla. |
| `CALL8 rel8` | `45` | signed PC-relatív offset | `( -- )` | Return address push és relatív branch. |
| `?DO8 rel8` | `46` | signed PC-relatív offset | `( limit start -- )` | Ha `start==limit`, loop exit; különben frame létrehozás. |
| `LOOP8 rel8` | `47` | signed PC-relatív offset | `( -- )` | Index +1; branch, amíg folytatódik, különben frame eltávolítás. |
| `+LOOP8 rel8` | `48` | signed PC-relatív offset | `( step -- )` | Index léptetése signed steppel. |
| `LEAVE8 rel8` | `49` | signed PC-relatív offset | `( -- )` | Loop frame eltávolítása és exit branch. |
| `PICK depth` | `4A` | unsigned depth | `( ... x ... -- ... x ... x )` | A depth-edik elemet TOS-ra másolja; depth 0 = TOS. |
| `ROLL depth` | `4B` | unsigned depth | változó | A depth-edik elemet TOS-ra mozgatja, közteseket eltolja. |

`4C..4F` a zero-page direct memóriaformák, `50` a system prefix. `51..7F` jelenleg kiosztatlan/fenntartott kétbájtos opcode-hely.

A relatív offset a displacement bájt fetch-e utáni `PC`-hez képest értendő. Az assembler automatikusan rövid relatív formát választ, ha a cél belefér.

## Hárombájtos utasítások

Az opcode után 16 bites little-endian immediate vagy abszolút cím következik.

| Mnemonik | Hex | Stack effect | Definíció |
|---|---:|---|---|
| `PUSH16 imm16` | `80` | `( -- value )` | 16 bites literál push. |
| `JMP addr16` | `81` | `( -- )` | Abszolút ugrás. |
| `JZ addr16` | `82` | `( flag -- )` | Flag pop; ugrás, ha nulla. |
| `JNZ addr16` | `83` | `( flag -- )` | Flag pop; ugrás, ha nem nulla. |
| `CALL addr16` | `84` | `( -- )` | Return address push és abszolút ugrás. |
| `?DO addr16` | `85` | `( limit start -- )` | Zero-trip loop setup; exit, ha `start==limit`. |
| `LOOP addr16` | `86` | `( -- )` | Index növelés és branch, amíg folytatódik. |
| `+LOOP addr16` | `87` | `( step -- )` | Signed step és branch, amíg folytatódik. |
| `LEAVE addr16` | `88` | `( -- )` | Frame eltávolítás és exit ugrás. |
| `LOAD8ABS addr16` *(assembler-generated)* | `89` | `( -- value )` | Unsigned byte read abszolút címről. |
| `LOAD16ABS addr16` *(assembler-generated)* | `8A` | `( -- value )` | 16 bites cella read abszolút címről. |
| `STORE8ABS addr16` *(assembler-generated)* | `8B` | `( value -- )` | Alsó byte store abszolút címre. |
| `STORE16ABS addr16` *(assembler-generated)* | `8C` | `( value -- )` | 16 bites cella store abszolút címre. |

`8D..BF` jelenleg kiosztatlan/fenntartott hárombájtos opcode-hely. `C0..FF` a négybájtos osztály és jelenleg teljesen fenntartott.

## Strukturált vezérlés assembler-viselkedése

Az assembler automatikusan költségorientált kódolást választ:

- `-1` és `0..10` egybájtos literálopcode;
- más, pozitív 8 bites érték `PUSH8`; signed 8 bites negatív `PUSHS8`; a többi `PUSH16`;
- lokális branch/call/loop transfer signed 8 bites relatív formát kap, ha a végleges displacement belefér; különben abszolút 16 bites formát;
- konstans-cím minták belső `LOAD8ABS`, `LOAD16ABS`, `STORE8ABS`, `STORE16ABS` gépi formára hajthatók a cím push + általános memória-primitív helyett;
- strukturált `DO` nestingből `EXIT` előtt az assembler beilleszti a szükséges `UNLOOP`-okat.

## Aritmetikai és memóriaszabályok

- A cellák 16 bitesek, az aritmetika modulo 65536 wrap-el.
- `DIV` és `MOD` unsigned és 0-val osztáskor trapet okoz.
- `SLT`, `SGT` és `0<` signed kétkomplementes 16 bites értelmezést használ.
- `SHL/SHR` a shift countot 15-tel maszkolja.
- A 16 bites memória-hozzáférések little-endian bájtsorrendűek.

## Pre-decrement memóriabejárók

A korábban kevésbé értékes 11..14 dedikált literálok helyén négy egybájtos opcode szolgál hátrafelé lineáris memóriabejárásra. A 11..14 literálok továbbra is normálisan assemblálódnak `PUSH8`-cal.

| Word | Opcode | Stack effect |
|---|---:|---|
| `C@-` / `LOAD8-` | `3C` | `( addr -- addr-1 value )` |
| `C!-` / `STORE8-` | `3D` | `( value addr -- addr-1 )` |
| `@-` / `LOAD16-` | `3E` | `( addr -- addr-2 value )` |
| `!-` / `STORE16-` | `3F` | `( value addr -- addr-2 )` |

## Return/control stack megjegyzés

A `DO...LOOP` frame-ek ugyanazt a return/control stacket használják, mint a call return address-ek. `I` és `J` annak a szónak a loopjaihoz készült, amelyik a loopot birtokolja. Hívott szó nem feltételezheti, hogy a hívó loop indexe közvetlenül `I/J`-vel elérhető. Ez a korlátozás elkerüli egy harmadik loop stack vagy dedikált loop-frame pointer költségét.

## Közvetlen zero-page formák

`0x4C..0x4F` kétbájtos `Load8Zp`, `Load16Zp`, `Store8Zp`, `Store16Zp` utasítás (opcode + address8). Az assembler normál esetben automatikusan választja, ha egy `0x00..0xFF` közötti konstans cím után `C@`, `@`, `C!` vagy `!` következik.

## Megszakításvezérlő system prefix

Az IRQ-vezérlés a szabad kétbájtos `50 xx` system prefixet használja a forró egybájtos opcode-hely fogyasztása helyett.

| Bájtok | Utasítás | Hatás |
|---|---|---|
| `50 00` | `EI` | maskable interruptok globális engedélyezése |
| `50 01` | `DI` | maskable interruptok globális tiltása |
| `50 02` | `IRET` | mentett interrupt-enable állapot és `PC` visszaállítása a return/control stackről |

Az assembler a sima `EI`, `DI`, `IRET` mnemonikát fogadja. Interrupt belépés a meglévő return/control stacket használja; nincs harmadik interrupt stack.

## Integer DSP kiterjesztés

| Utasítás | Hex kódolás | Stack effect |
|---|---|---|
| `ASR1` | `50 03` | `( x -- x/2 )`, signed aritmetikai shift |
| `MULQ15` | `50 04` | `( a b -- q15(a*b) )` |

A `MULQ15` signed 16 bites operandusokat, 32 bites köztes értéket és aritmetikai `>>15`-öt használ; a `0x8000 * 0x8000` egyedi túlcsordulás `0x7FFF`-re telítődik.

## Külön videó-címtér system kiterjesztések

A stack gép a sűrű egybájtos opcode-teret úgy őrzi meg, hogy a videómemória-műveleteket `SYS` (`0x50`) + subopcode formában kódolja. A videócímek 16 bites offsetek a külön videó-adattérben.

| Mnemonik | Hex | Stack effect |
|---|---|---|
| `VC@` | `50 10` | `( addr -- value )` |
| `V@` | `50 11` | `( addr -- value )` |
| `VC!` | `50 12` | `( value addr -- )` |
| `V!` | `50 13` | `( value addr -- )` |
| `VC@+` | `50 14` | `( addr -- addr+1 value )` |
| `V@+` | `50 15` | `( addr -- addr+2 value )` |
| `VC!+` | `50 16` | `( value addr -- addr+1 )` |
| `V!+` | `50 17` | `( value addr -- addr+2 )` |
| `VC@-` | `50 18` | `( addr -- addr-1 value )` |
| `V@-` | `50 19` | `( addr -- addr-2 value )` |
| `VC!-` | `50 1A` | `( value addr -- addr-1 )` |
| `V!-` | `50 1B` | `( value addr -- addr-2 )` |

## Többwordös integer segédutasítások

A Stack ISA egyetlen minimális `C` carry/borrow állapotot tart fenn a többwordös aritmetikához; az összehasonlítások továbbra is stackértéket termelnek, nincs általános status-register modell.

- `ADD` (`10`): `( a b -- r )`, `C` = carry-out.
- `SUB` (`11`): `( a b -- r )`, `C=1` = no borrow.
- `SHL1` (`1E`): `( a -- r )`, régi bit15 -> `C`.
- `SHR1` (`1F`): `( a -- r )`, régi bit0 -> `C`.
- `ADC` (`50 06`): `( a b -- r )`, `r=a+b+C`, frissíti `C`-t.
- `SBC` (`50 07`): `( a b -- r )`, `r=a-b-(1-C)`, frissíti a no-borrow `C`-t.
- `RCR1` (`50 08`): `( a -- r )`, régi `C` -> bit15, régi bit0 -> `C`.
- `UMUL` / `MUL32` (`50 05`): `( a b -- lo hi )`, teljes unsigned 16x16 -> 32 bites szorzat.
