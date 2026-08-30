# SVM ISA képesség- és ár–érték mátrix

## 1. Cél

Ez a dokumentum a kilenc SVM CPU-t ugyanazon képességek mentén hasonlítja össze. Nem az a cél, hogy az architektúrák egymásra hasonlítsanak, hanem hogy minden gép csak azokat a primitíveket tartsa meg, amelyek a saját operandusmodelljében jó ár–értéket adnak.

A státuszjelölések:

- **K** – kötelező: a közös platform vagy az SVM-C szemantikája miatt szükséges;
- **A** – ár–értékben indokolt: nem feltétlenül minimális, de mérhetően csökkenti a kódméretet, memóriaforgalmat vagy compiler-spillt;
- **M** – mérlegelendő: értéke valószínű, de statikus kód-/használatelemzéssel kell igazolni;
- **E** – elhagyásra/aliasra jelölt: ugyanaz a szemantika olcsón előállítható más primitívből;
- **—** – az adott operandusmodellben szándékosan nincs vagy nem értelmezhető.

## 2. Architektúra-spektrum

| CPU | Operandusmodell | Fő összehasonlítási kérdés |
|---|---|---|
| Stack | 0-címes | mennyit érnek az implicit operandusok és a kódsűrűség a stack-forgalomért cserébe? |
| Accumulator | 1-címes | mennyire olcsó egyetlen implicit adatregiszter és két címregiszter? |
| MemReg | working-register + file memory | mennyit ér a W+direct-file modell és a kis hot file tér? |
| Register | 2-címes register-register | mennyit ér a tömör GPR-kódolás és a kompakt regiszterrészhalmaz? |
| Load/Store | 3-címes strict RISC | mennyit ér az egyszerű datapath és a compilerbarát háromoperandusos ALU? |
| Register-Memory | 2-címes GPR + memory source | mennyi explicit load és temp regiszter takarítható meg egyetlen memória-source ALU-val? |
| Memory-to-Memory | memória–memória | mennyi kód takarítható meg közvetlen memóriaoperandusokkal, és mennyibe kerül a descriptor/dekóder? |
| Belt16 | implicit-result belt | mennyi explicit adatmozgatás takarítható meg a legutóbbi eredmények relatív hivatkozásával? |

A kilenc gép együtt lefedi a klasszikus egyszerű 0-, 1-, 2- és 3-címes szervezéseket, a working-register és memory-to-memory modelleket, valamint az implicit-result belt elvet. A Belt16 új mérési kérdése az explicit temporary destinationök elhagyása; A TTA16 azóta bekerült, mert a MOVE-központú, transport-triggered modell valóban új végrehajtási alapelvet ad. OISC/VLIW továbbra sem indokolt az ár–érték célhoz.

## 3. Közös szemantikai képességek

| Képesség | Stack | Accumulator | MemReg | Register | Load/Store | Reg-Mem | Mem-Mem |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| 16 bites add/sub | K | K | K | K | K | K | K |
| AND/OR/XOR | K | K | K | K | K | K | K |
| változó SHL/SHR | K | K | K | K | K | K | K |
| `MUL` low16 | K | K | K | K | K | K | K |
| unsigned `DIV` | K | K | K | K | K | K | K |
| unsigned `MOD` | K | K | K | K | K | K | K |
| compare + conditional control flow | K | K | K | K | K | K | K |
| 8 bites zero-extending load | K | K | K | K | K | K | K |
| 16 bites load/store | K | K | K | K | K | K | K |
| külön VRAM load/store | K | K | K | K | K | K | K |
| `CALL/RET` | K | K | K | K | K | K | K |
| `EI/DI/IRET` | K | K | K | K | K | K | K |
| `ASR1` | A | A | A | A | A | A | A |
| `MULQ15` | A | A | A | A | A | A | A |

Megjegyzés: az `ASR1` és `MULQ15` nem általános C-követelmény, hanem a közös integer-DSP/FFT vizsgálat része. Azért **A**, nem **K**.

## 4. Kódsűrűségi primitívek

| Primitív | Stack | Accumulator | MemReg | Register | Load/Store | Reg-Mem | Mem-Mem |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| rövid relatív branch | A | A | A | A | A | A | A |
| rövid relatív call | A | A | A | A | A | A | A |
| assembler relaxation | A | A | A | A | A | A | A |
| rövid kis konstans | A | M | M | M | A (`LDI6`) | A (descriptor) | A (descriptor) |
| zero-page/direct rövid forma | A | A | A | A | — | A | A |
| egybyte-os `INC/DEC` | A | A | A | A | A | A | descriptor/ALU alapján |
| compact/hot regiszter vagy file részhalmaz | stack implicit | — | **A/M** | A | — | — | hot direct memória |
| külön `SUBI` opcode | — | **A** | **A** | **A** | hosszú immediate | descriptorból | descriptorból |

### 4.1. `SUBI`

A puszta numerikus eredmény szempontjából `SUBI imm16` helyettesíthető lenne `ADDI (-imm16 mod 65536)` formával, de flag-es architektúrán a carry/no-borrow szemantika eltérhet. Emiatt a Register ISA v3-ban a hardveres `SUBI` megmarad. Accumulator és MemReg esetén is csak akkor szabad aliasra váltani, ha a flag-szemantikát vagy az ABI-t külön rendezik.

A Stack gépen a `SUB` valódi kétoperandusos stack művelet, ezért ez a megállapítás nem vonatkozik rá.

### 4.2. Kis konstansok

A Stack literal `-1..10` formái és a Load/Store `LDI6` nagy értékűek, mert loop counter, null/true/false, kis index és bitmező esetén sok fetch byte-ot takarítanak meg.

Accumulator/Register/MemReg gépen egy külön rövid-immediate család csak akkor indokolt, ha a fordító- és példakód-vizsgálat szerint a hárombyte-os `LDAI/MOVI/LDI` érdemi részt képvisel. Ezt előbb mérni kell; szabad opcode-hely önmagában nem érv.

## 5. Lineáris memóriajárás

| Képesség | Stack | Accumulator | MemReg | Register | Load/Store | Reg-Mem | Mem-Mem |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| post-increment byte | A | A | A | A | **M** (szándékosan nincs) | A | A |
| post-increment word | A | A | A | A | **M** (szándékosan nincs) | A | A |
| pre-decrement byte | A | A | A | A | **M** (szándékosan nincs) | A | A |
| pre-decrement word | A | A | A | A | **M** (szándékosan nincs) | A | A |
| két független pointer természetesen | stack értékek | X+Y | FSR0+FSR1 | GPR | GPR | GPR | A0..A3 |

A Load/Store gépen az auto-update hiánya **nem hiányosság**, hanem mérési kontrollpont. Csak akkor szabad hozzáadni, ha a memcpy/memmove/VRAM fordító- és példakód-vizsgálat szerint a pointerfrissítés költsége aránytalanul dominál, és ezt a specifikáció célja mellett is el akarjuk rejteni.

## 6. Stack-specifikus primitívek

| Utasítás/család | Státusz | Indoklás |
|---|---|---|
| `DUP`, `DROP`, `SWAP`, `OVER`, `ROT` | A | alap stack-adatmozgatás; pótlásuk gyorsan több utasítást és spillt igényel |
| `NIP`, `TUCK`, `2DUP`, `2DROP` | A | **assembly-orientált convenience**: a C backend nem igényli, de kézi stack/Forth kódban jó kódsűrűséget és olvashatóságot ad; megtartandó |
| `PICK`, `ROLL` | A | **assembly-orientált deep-stack hozzáférés**: a C backend nem igényli, kézi assemblyben csökkenti a temporális memóriahasználatot; megtartandó |
| `DO/?DO/I/J/LOOP/+LOOP/LEAVE/UNLOOP` | A | **assembly-orientált structured-loop blokk**: nem C-követelmény, hanem a stack/Forth programozási modell tudatos része; megtartandó |
| külön összehasonlító utasítások (`=`, `<>`, `U<`, stb.) | A | egybyte-os boolean termelés; általános SUB+flag emuláció stacken kód- és stack-forgalmat növelne |

**Megjegyzés:** a `NIP/TUCK/2DUP/2DROP/PICK/ROLL` megtartását nem a C backend használati gyakorisága dönti el. Ezek tudatosan assembly-orientált primitívek: kézi stack/Forth programok olvashatóságát és kódsűrűségét szolgálják. Csak akkor érdemes később extensionbe tolni őket, ha az opcode-tér tényleges szűkössége ezt kikényszeríti.

## 7. Accumulator-specifikus primitívek

| Utasítás/család | Státusz | Indoklás |
|---|---|---|
| `TAX/TXA`, `TAY/TYA` | A | az A és pointerregiszterek közti explicit adatút alapvető |
| X és Y pointerregiszter | A | memcpy/memmove, két tömb, src/dst természetes kezelése |
| `PUSHX/POPX` | M | C-ben kevésbé gyakori, de függvény/pointer-preservation esetén olcsó |
| X ALU (`ADDX`, `SUBX`, ... ) | A | accumulator modellben ez adja a második operandust memória nélkül |
| Y ALU | — | szándékosan nincs; Y címregiszter, nem második teljes ALU operandus |
| X/Y post-inc/pre-dec load/store | A | lineáris memóriajárásnál nagyon jó kódméret/memóriaforgalom arány |
| `SUBI` | A | carry/no-borrow szemantika miatt natív forma indokolt |

Az X és Y közötti aszimmetria jó ár–érték kompromisszum: két pointer kell, de nem kell két teljes második ALU operandusút.

## 8. MemReg-specifikus primitívek

| Utasítás/család | Státusz | Indoklás |
|---|---|---|
| W working register | K | az architektúra definíciója |
| 8 bites direct file tér | A | kétbyte-os hot statikus adatkezelés |
| FSR0/FSR1 | A | teljes 64 KiB elérés és két pointer |
| direct `f,W` / `f,F` ALU | A | a modell fő értéke; `f,F` memória write-backet takarít meg |
| 0x00..0x0F egybyte-os hot MOV | **A** | compiler statikusok első része közvetlenül profitál |
| 0x00..0x0F egybyte-os hot ADD/AND | **A** | a wide-int/soft-float maszkolási minták miatt véglegesített kiosztás |
| `SUBI` | A | carry/no-borrow szemantika miatt natív forma indokolt |
| W push/pop | A | expression evaluation/funkciók miatt |

### 8.1. MemReg compiler-owned hot scratch — átvezetve

A korábbi külön C-fordítók MemReg backendje az általános bináris kifejezések 16 bites ideiglenes jobb operandusát a `0xFE` file címre írta. Ez kívül esett az egybyte-os `0x00..0x0F` hot tartományon.

A compiler most a `0x000E..0x000F` két bájtot saját 16 bites scratch-területként foglalja le. A felhasználói statikusok továbbra is használhatják a `0x0000..0x000D` tartományt, az allocator pedig szükség esetén `0x0010`-től folytatja. A többi CPU célpont memóriaelrendezése nem változik.

Tipikus kód:

```asm
MOV16 W, 0x0E
POPW
ADD 0x0E,W
```

Az első és az `ADD`/`AND` utasítás így hot egybyte-os formát kaphat. A többi ALU művelet legalább a scratch-be író `MOV16` rövidül. A hot XOR helyét az AND vette át, mert a wide-int/soft-float kód gyakrabban maszkol, mint XOR-ol.

### 8.2. Hot `AND` döntés

A hot tér `ADD` és `AND` műveletet gyorsít. Az `AND` a wide-int és soft-float könyvtárak exponent-, mantissza- és előjelmaszkjai miatt gyakrabban hasznos, mint az XOR. Az XOR továbbra is normál ALU-formában teljes értékűen elérhető.

## 9. Register-specifikus primitívek

| Utasítás/család | Státusz | Indoklás |
|---|---|---|
| R0..R7 GPR | K | architektúra alapja |
| R0..R3 compact subset | A | hot belső ciklusoknál 1 byte kétregiszteres ALU/load/store |
| egybyte-os unary R0..R7 | A | kevés dekóderköltség, jó kódsűrűség |
| compact `MOV/ADD/SUB/CMP/AND` | A/M | Register ISA v3 kiosztás; XOR normál formában marad |
| compact byte load/store | A | C `u8` és karakter/MMIO használat miatt |
| general word load/store | K | 16 bites adatmodell |
| post-inc/pre-dec walker | A | memcpy/memmove és VRAM |
| zero-page implicit R0 formák | A | compiler expression/result regiszterrel jól illeszkedik |
| `SUBI` | A | hardveres immediate forma megmarad a helyes C flag miatt |

A Register ISA v3-ban a compact `B0..BF` család `AND`, nem `XOR`; a MemReg `E0..FF` hot tartománya szintén `AND`-et gyorsít. Az XOR mindkét gépen megmarad normál ALU-műveletként. Ez a wide-int/soft-float maszkolási minták miatt jobb kódsűrűségi kiosztás.

## 10. Load/Store-specifikus primitívek

| Utasítás/család | Státusz | Indoklás |
|---|---|---|
| háromoperandusos reg-reg ALU | K | kontroll-RISC fő tulajdonság |
| memória csak LD/ST útján | K | strict load/store definíció |
| `LDI6` | A | kis konstansok olcsó előállítása |
| `DIVU/MODU` | K | SVM-C szemantika |
| `MULQ15`, `ASR1` | A | közös DSP mérés |
| `SEXT8` | E/eltávolítva | `i8` már létezik, de az előjelkiterjesztés jelenleg olcsó szoftveres/library loweringgel kezelhető; külön opcode csak akkor indokolt, ha a backend természetesen és gyakran használná |
| auto-increment LD/ST | **M/szándékosan nincs** | a strict load/store modell tisztaságát tartjuk; csak egyértelmű statikus kódgenerálási előny esetén érdemes újragondolni |
| scaled/indexed addressing | — | túl sok címképző/dekóder-költség |

A v0.3 jelenleg jó implementációs cél. Új primitívet statikus indoklás nélkül nem javasolt hozzáadni.

## 11. Register-Memory-specifikus primitívek

| Utasítás/család | Státusz | Indoklás |
|---|---|---|
| GPR destination + reg/mem/imm source | K | architektúra fő kérdése |
| memória-source ALU | K | ez különíti el a Register/Load-Store géptől |
| memória destination ALU | — | ez már Memory-to-Memory irány lenne |
| source descriptor | A | közös dekóderrel reg/mem/imm formák |
| post-inc/pre-dec csak load/store | A | lineáris járás mellékhatásos ALU nélkül |
| byte ALU | — | `LD8 -> word ALU -> ST8` elegendő |
| 8 GPR | A | az implementált v1 döntése; közvetlenebb összevetés a Register/Load-Store géppel és kevesebb mesterséges spill |

A korábbi 4 GPR-os tervezet helyett az implementált v1 8 GPR-t használ. Ezt most stabil architekturális döntésként kezeljük; visszaszűkítés csak egyértelmű összköltség-előny esetén indokolt.

## 12. Memory-to-Memory-specifikus primitívek

| Utasítás/család | Státusz | Indoklás |
|---|---|---|
| memória destination + memória/regiszter nélküli source descriptor | K | architektúra fő tulajdonság |
| A0..A3 címregiszter | A | pointeres C különben irreálisan drága lenne |
| hot direct `0x00..0x7F` | A | descriptor-byte megtakarítás |
| általános absolute descriptor | K | teljes 64 KiB elérés |
| immediate source descriptor | A | külön `...I` opcode-családok helyett |
| post-inc/pre-dec pointer descriptor | A | memcpy/memmove |
| külön `VSTI8/VSTI16` | E/eltávolítva | normál VST immediate source-szal lefedhető |
| általános adat-GPR | — | szándékosan nincs; különben Reg-Mem/MemReg felé csúszna |

A v0.3 specifikációban jelenleg nincs további nyilvánvaló redundáns primitív.

## 13. Kereszt-ISA optimalizációs lista

### P0 – biztos / már eldöntött

1. **`SUBI`:** flag-szemantika miatt nem tekintjük automatikusan eltávolíthatónak; a Register ISA v3-ban hardveres marad.
2. **Load/Store `SEXT8`:** maradjon reserved; `i8` létezik, de a jelenlegi soft/lowering megoldás mellett külön opcode nem indokolt.
3. **Memory-to-Memory VST immediate:** egyetlen általános VST source descriptor, külön VSTI nélkül.

### P1 – statikus használatelemzéssel felülvizsgálandó

1. **MemReg compiler hot scratch:** elkészült; `0x000E..0x000F` lefoglalt 16 bites compiler scratch.
2. **MemReg hot ALU műveletmix:** lezárva, `ADD/AND`; az XOR normál formában marad.
3. **Register compact ALU műveletmix:** lezárva, `MOV/ADD/SUB/CMP/AND`; az XOR normál formában marad.
4. **Stack macro-primitívek:** `NIP/TUCK/2DUP/2DROP/PICK/ROLL` használata a compilerben és kézi assemblyben.
5. **Load/Store auto-update hiánya:** pointer update instruction count a lineáris reprezentatív programokon; elsődlegesen mérési kontroll, nem automatikus bővítési jelölt.

### P2 – csak később, ha a statikus kódvizsgálat indokolja

- rövid immediate család Accumulator/Register/MemReg gépen;
- Stack ritka műveletek prefix extensionbe mozgatása;
- hot/compact opcode-családok újraelosztása.

## 14. Egységes statikus felülvizsgálati szempontok

Minden CPU-n és minden reprezentatív programon ugyanazokat kell gyűjteni:

1. executable kódméret byte-ban;
2. statikus utasításszám;
3. dinamikusan végrehajtott utasításszám;
4. instruction-fetch byte/ciklus;
5. CPU-adatmemória 8/16 bites olvasások és írások;
6. VRAM olvasások és írások;
7. stack/control-stack memóriaforgalom;
8. explicit spill/reload darabszám;
9. branch/call darabszám és rövid/hosszú forma aránya;
10. compiler által használt opcode/mnemonic gyakoriság;
11. kézi assembly reprezentatív programok opcode/mnemonic gyakorisága;
12. teljes determinisztikus VM-ciklusszám.

A hardver/implementációs költséget külön, kvalitatív pontozással kell mellétenni:

- programmer-visible state mennyisége;
- címképző utak száma;
- egy utasítás maximális adatmemória-hozzáférése;
- változó hosszúságú descriptor/dekóder bonyolultsága;
- speciális végrehajtó primitivek száma.

## 15. Reprezentatív ellenőrző programkészlet

Minimum:

- scalar arithmetic és comparison;
- kis `for`/`while` ciklus;
- function call + paraméterek + lokálisok;
- `memset`;
- forward `memcpy`;
- overlapping backward `memmove`;
- `u8` tömbfeldolgozás;
- `u16` tömbfeldolgozás;
- framebuffer fill;
- VRAM copy/plot;
- text/MMIO output;
- `fft_q15`;
- legalább egy kézzel optimalizált assembly kernel CPU-nként.

A kézi assembly azért kötelező, mert egy compiler pillanatnyi hiányossága nem bizonyítja, hogy egy ISA-primitív értéktelen.

## 16. Döntési szabály

Utasítást vagy kódolási családot akkor jelöljünk végleges eltávolításra, ha az alábbiak közül legalább kettő teljesül:

1. compiler és reprezentatív kézi assembly sem használja érdemben;
2. rövid, hasonló memóriaforgalmú utasítássorral helyettesíthető;
3. eltávolítása tényleges dekóder-, datapath-, állapot- vagy jelentős opcode-terület-költséget csökkent.

Fordítva: pusztán azért nem adunk új utasítást, mert van szabad opcode. Egy új primitívnek legalább reprezentatív kódban egyértelmű kódméret-, ciklus- vagy memóriaforgalom-előnyt kell adnia.

## 17. Következő gyakorlati lépés

A mátrix után a prioritás nem külön benchmark-telemetria, hanem a statikusan igazolható kódgenerálási egyszerűsítés: redundáns opcode helyett alias, meglévő immediate/memory-source forma jobb kihasználása, valamint assembler-oldali branch relaxation. Külön mérőrendszer nem része a célplatformnak.

Az új Load/Store, Register-Memory és Memory-to-Memory CPU implementálása során ugyanezt a mérési interfészt kell használni; így a későbbi utasításkészlet-takarítás nem benyomás, hanem közvetlen összehasonlító adat alapján történik.


## Többwordös integer támogatás

A flag-es architektúrák `ADC/SBC/MULHU/RCR1` jellegű primitíveket használnak. A Stack architektúra is kapott egyetlen minimális `C` carry/borrow állapotot a többwordös aritmetikához: `ADD/SUB/SHL1/SHR1` frissíti, `ADC/SBC/RCR1` használja. Az összehasonlítások továbbra is stackértéket termelnek, tehát nincs általános flags-regiszter. Az `UMUL (a b -- lo hi)` teljes szorzat megmarad. Ez az `i32/u32`, 32x32->64 és soft-`f16/f32` könyvtárak költségét csökkenti anélkül, hogy 32 bites ALU vagy FPU kerülne a CPU-kba.


## Belt16-specifikus primitívek

| Primitív | Státusz | Indoklás |
|---|---|---|
| `b0..b7` eredményszalag | K | az architektúra definíciója; nincs GPR vagy operandusstack |
| implicit `b0` destination | K | megszünteti az explicit eredményregiszter kódolását |
| `PASS bN` | A | olcsó belt-élettartam/duplikáció assemblyben |
| `PUSH bN` / `POP` | A | C temporális spill és kézi assembly; nem általános register file |
| `ADC/SBC/MULHU/RCR1` | A | ugyanaz a többwordös integer/soft-float indok, mint a flag-es gépeken |
| külön belt-lifetime compiler optimalizáció | M | backend-optimalizáció, nem ISA-bővítés |


### TTA16

A TTA16 explicit adattranszporttal dolgozik: a funkcionális egység triggerportjára történő `MOV` indítja a műveletet. Ez külön tengely a regiszter-, stack- és belt-modellekhez képest.
