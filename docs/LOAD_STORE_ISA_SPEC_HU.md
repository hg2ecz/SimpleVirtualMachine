# SVM Load/Store CPU – ISA specifikáció (implementált v1)

## 1. Cél

A Load/Store CPU a közös SVM platform tiszta RISC összehasonlítási pontja. A gép fő tervezési elve, hogy **aritmetikai/logikai utasítás közvetlenül soha nem ér el memóriát**: a rendszermemória és a külön VRAM kizárólag explicit load/store utasításokkal érhető el.

A cél nem egy létező RISC processzor másolása, hanem egy kis költségű, fordítóbarát kontrollarchitektúra létrehozása a Register, Stack, Accumulator és MemReg gépek mellé.

## 2. Közös platformprofil

A CPU a meglévő SVM platformot változtatás nélkül használja:

- 16 bites little-endian adatmodell;
- 16 bites PC és 64 KiB CPU-címtér (`0x0000..0xFFFF`);
- MMIO a közös platformdokumentáció szerint, jelenleg `0xFF00..0xFF2A`;
- nincs guest-visible System ROM;
- külön 16 KiB, adat-only VRAM (`0x0000..0x3FFF`), ebből 16 000 bájt framebuffer;
- ugyanaz a karaktergenerátor, paletta, konzol, timer és IRQ modell;
- az utasítás-fetch kizárólag a CPU-címtérből történik.

A pontos platformviselkedésre a `PLATFORM_HU.md` az irányadó.

## 3. Tervezési alapelvek

1. **Strict load/store:** ALU csak regiszteroperandusokon dolgozik.
2. **Háromoperandusos ALU:** `Rd = Ra op Rb`, így a forrásoperandusok nem sérülnek.
3. **8 általános 16 bites regiszter:** elegendő a C fordítóhoz, de nem növeli indokolatlanul a virtuális hardverköltséget.
4. **16 bites, szóigazított alapkódolás:** minden utasítás legalább egy 16 bites instruction word; hosszú immediate/abszolút utasítások egy további 16 bites extension wordöt használnak.
5. **Kevés címzési mód:** `base + signed offset`, illetve külön hosszú konstans/vezérlés.
6. **Nincs implicit memóriaoperandus, nincs memória–memória ALU.**
7. **A VRAM ugyanezt a címképzést használja, de külön video load/store major opcode-családdal.**
8. A kihasználatlan opcode-terület fenntartott; nem cél minden kódpont kitöltése.

## 4. Programmer-visible állapot

- `R0..R7`: nyolc 16 bites általános regiszter.
- `PC`: 16 bites program counter.
- flagek: `Z`, `N`, `C`.
- globális interrupt-enable állapot.

Egyetlen GPR sem hardwired zero. Az implementált ABI azonban `R6`-ot egységes stack pointerként tartja fenn: resetkor `R6=0xFF00`, a verem lefelé nő a `0xFB00..0xFEFF` RAM-tartományban. A `CALL/RET`, interrupt entry/`IRET`, valamint az assembler `PUSH/POP` kényelmi formái ugyanazt az `R6`-ot használják; nincs külön rejtett hardver-SP.

## 5. Instruction word és mezők

Minden instruction word 16 bites, little-endian bájtsorrendben tárolva és 2 bájtos határra igazítva.

Általános jelölés:

- `M`: bits `15..12`, 4 bites major opcode;
- `Rd/Ra/Rb/Rs`: 3 bites regiszterszám;
- `fn`: műveletkód;
- `imm6`: 6 bites immediate;
- `off6`: signed 6 bites byte offset (`-32..+31`);
- `rel9`: signed 9 bites **instruction-word** relatív eltolás;
- `ext16`: a következő 16 bites szó.

Az extension wordöt használó utasítás teljes hossza 4 bájt.

## 6. Major opcode térkép

| Major | Család | Alak |
|---:|---|---|
| `0` | system/special | `0000 sub12` |
| `1` | ALU3 | `0001 fn3 Rd3 Ra3 Rb3` |
| `2` | unary/compare/move | `0010 fn3 Rd3 Ra3 xxx3` |
| `3` | small-immediate ALU | `0011 fn3 Rd3 imm6` |
| `4` | `LD8` | `0100 Rd3 Ra3 off6` |
| `5` | `LD16` | `0101 Rd3 Ra3 off6` |
| `6` | `ST8` | `0110 Rs3 Ra3 off6` |
| `7` | `ST16` | `0111 Rs3 Ra3 off6` |
| `8` | relative branch | `1000 cond3 rel9` |
| `9` | long immediate | `1001 fn3 Rd3 000000`; + `ext16` |
| `A` | long control flow | `1010 fn3 000000000`; + `ext16` |
| `B` | video load/store | `1011 fn2 Rv3 Ra3 off4` |
| `C` | integer DSP | kijelölt formák |
| `D..F` | reserved | trap, amíg nincs specifikálva |

A reserved encoding végrehajtási hibát okoz; nem kezelhető NOP-ként.

## 7. ALU3 – major `1`

Formátum: `0001 fn3 Rd3 Ra3 Rb3`.

| `fn` | Mnemonic | Szemantika |
|---:|---|---|
| `0` | `ADD Rd,Ra,Rb` | `Rd = Ra + Rb` |
| `1` | `SUB Rd,Ra,Rb` | `Rd = Ra - Rb` |
| `2` | `AND Rd,Ra,Rb` | bitenkénti AND |
| `3` | `OR Rd,Ra,Rb` | bitenkénti OR |
| `4` | `XOR Rd,Ra,Rb` | bitenkénti XOR |
| `5` | `MUL Rd,Ra,Rb` | alsó 16 bit |
| `6` | `SHL Rd,Ra,Rb` | `Ra << (Rb & 15)` |
| `7` | `SHR Rd,Ra,Rb` | logikai jobbra shift |

Az aritmetikai/logikai eredmény frissíti `Z,N`-t. `ADD/SUB` frissíti `C`-t; bitműveletek és shift nem módosítják `C`-t. `MUL` `C=0` értéket ad.

## 8. Unary, compare és move – major `2`

Formátum: `0010 fn3 Rd3 Ra3 xxx3`; az alsó három bit 0 kell legyen.

| `fn` | Mnemonic | Szemantika |
|---:|---|---|
| `0` | `MOV Rd,Ra` | `Rd = Ra` |
| `1` | `CMP Ra,Rd` | flagek `Ra-Rd` alapján, nincs write-back |
| `2` | `NOT Rd,Ra` | `Rd = ~Ra` |
| `3` | `NEG Rd,Ra` | `Rd = 0-Ra` |
| `4` | `ASR1 Rd,Ra` | aritmetikai jobbra shift 1 |
| `5..7` | reserved | trap |

`CMP` esetén `C=1` jelentése: nincs borrow.

## 9. Small-immediate ALU – major `3`

Formátum: `0011 fn3 Rd3 imm6`.

Az `imm6` az `ADDI/CMPI` műveleteknél signed `-32..+31`, a logikai műveleteknél és `LDI6` esetén zero-extended `0..63`.

| `fn` | Mnemonic |
|---:|---|
| `0` | `ADDI Rd,simm6` |
| `1` | `LDI6 Rd,uimm6` |
| `2` | `CMPI Rd,simm6` |
| `3` | `ANDI Rd,uimm6` |
| `4` | `ORI Rd,uimm6` |
| `5` | `XORI Rd,uimm6` |
| `6` | `SHLI Rd,uimm4` |
| `7` | `SHRI Rd,uimm4` |

Shift esetén csak az immediate alsó 4 bitje jelentős; a felső két bitnek nullának kell lennie.

`LDI6` közvetlenül kis pozitív konstans előállítására szolgál. Ez különösen fontos, mert nincs hardwired zero regiszter. A `SUBI Rd,k` külön hosszú-immediate dekódot használ (`SUBI16` szemantika), mert az `ADDI Rd,-k` ugyanazt a numerikus eredményt adná, de a `C` carry/no-borrow flaget nem minden esetben azonosan állítaná. Kis, 6 bites külön `SUBI` kódpont továbbra sem kell; a kivonás mindig a hosszú immediate családon megy.

## 10. CPU-memória load/store – major `4..7`

Címképzés minden esetben:

`EA = Ra + sign_extend(off6)` modulo 65536.

| Major | Mnemonic | Szemantika |
|---:|---|---|
| `4` | `LD8 Rd,[Ra+off6]` | zero-extended byte load |
| `5` | `LD16 Rd,[Ra+off6]` | little-endian 16 bites load |
| `6` | `ST8 [Ra+off6],Rs` | `low8(Rs)` tárolása |
| `7` | `ST16 [Ra+off6],Rs` | 16 bites store |

A load/store nem módosít flaget. Unaligned 16 bites hozzáférés megengedett, a közös platform memóriaszemantikáját követi.

Nincs abszolút memóriacímzés külön utasításban: egy abszolút cím először `LDI`-vel regiszterbe kerül. Ez szándékos RISC-költség.

## 11. Relatív branch – major `8`

Formátum: `1000 cond3 rel9`.

A cél:

`PC = next_pc + sign_extend(rel9) * 2`.

| `cond` | Mnemonic | Feltétel |
|---:|---|---|
| `0` | `BRA rel9` | mindig |
| `1` | `BZ rel9` | `Z=1` |
| `2` | `BNZ rel9` | `Z=0` |
| `3` | `BC rel9` | `C=1` |
| `4` | `BNC rel9` | `C=0` |
| `5` | `BN rel9` | `N=1` |
| `6` | `BNN rel9` | `N=0` |
| `7` | reserved | trap |

A branch tartomány `-512..+511` instruction word, azaz `-1024..+1022` bájt a következő utasítástól.

## 12. Long immediate – major `9`

Az első word után egy `ext16` következik.

| `fn` | Mnemonic | Szemantika |
|---:|---|---|
| `0` | `LDI Rd,imm16` | `Rd=imm16` |
| `1` | `ADDI16 Rd,imm16` | `Rd+=imm16` |
| `2` | `CMPI16 Rd,imm16` | compare, csak flag |
| `3` | `SUBI16 Rd,imm16` | `Rd-=imm16`, `C` = no-borrow |
| `4..7` | reserved | trap |

Az assembler automatikusan a major `3` rövid immediate formát választja, ha az érték belefér és a szemantika azonos.

## 13. Long control flow – major `A`

Az `ext16` abszolút CPU-cím.

| `fn` | Mnemonic |
|---:|---|
| `0` | `JMP addr16` |
| `1` | `CALL addr16` |
| `2` | `JZ addr16` |
| `3` | `JNZ addr16` |
| `4` | `JC addr16` |
| `5` | `JNC addr16` |
| `6` | `JN addr16` |
| `7` | `JNN addr16` |

A v1 assembler a hosszú `JMP/Jcc/CALL` formákat stabilan generálja. A major `8` relatív branch decode implementált, és explicit `BRA/BZ/BNZ/BC/BNC/BN/BNN` displacement formában elérhető. Automatikus branch relaxation még nincs; ez későbbi assembler-oldali optimalizáció.

## 14. Külön VRAM load/store – major `B`

Formátum: `1011 fn2 Rv3 Ra3 off4`, ahol `off4` signed `-8..+7` byte offset.

| `fn` | Mnemonic |
|---:|---|
| `0` | `VLD8 Rv,[Ra+off4]` |
| `1` | `VLD16 Rv,[Ra+off4]` |
| `2` | `VST8 [Ra+off4],Rv` |
| `3` | `VST16 [Ra+off4],Rv` |

A cím a külön video-adatcímtérben értendő. CPU RAM és VRAM nem aliasolható.

Lineáris framebuffer-járáshoz nincs külön post-increment opcode: a fordító `VLD/VST` + `ADDI` párost használ. Ennek költsége a RISC összehasonlítás része.

## 15. System/special – major `0`

A `sub12` értékek kezdeti kiosztása:

| `sub12` | Mnemonic |
|---:|---|
| `000` | `NOP` |
| `001` | `HALT` |
| `002` | `RET` |
| `003` | `EI` |
| `004` | `DI` |
| `005` | `IRET` |
| `006..FFF` | reserved |

`CALL/RET` és interrupt entry az ABI szerinti `R6` stack pointert használja; nincs külön rejtett hardver-SP. A mag ISA-ban nincs dedikált `PUSH/POP` opcode: az assembler `PUSH/POP` kényelmi formái `R6` módosítására és normál `LD/ST` műveletekre bomlanak. Ez megőrzi a strict load/store elvet, miközben a vezérlési és compiler-temporary verem ugyanazon, ütközésmentes mutatót használja.

## 16. Arithmetic extension – major `C`

A meglévő SVM-C nyelvi részhalmaz teljes lefedéséhez a futásidejű osztás és maradékképzés is szükséges. A major `C` ezért nem csak DSP-család, hanem ritkább, drágább aritmetikai műveletek szabályos háromoperandusos kódtere.

Formátum: `1100 fn3 Rd3 Ra3 Rb3`.

| `fn` | Mnemonic | Szemantika |
|---:|---|---|
| `0` | `DIVU Rd,Ra,Rb` | unsigned 16 bites osztás |
| `1` | `MODU Rd,Ra,Rb` | unsigned 16 bites maradék |
| `2` | `MULQ15 Rd,Ra,Rb` | signed Q15 szorzás, 32 bites köztes eredmény, `>>15`, `0x8000*0x8000 -> 0x7FFF` |
| `3..7` | reserved | trap |

`DIVU/MODU` nulla osztó esetén trapet okoz. `Z,N` az eredményből frissül, `C=0`. Az `ASR1` továbbra is major `2` alatt érhető el.

Ez a három művelet elegendő a natív 8/16 bites SVM-C aritmetikához és a Q15 könyvtári/példakódhoz; a 32/64 bites és lebegőpontos műveletek szoftverkönyvtárban maradnak.

## 17. C ABI javaslat

A backend kezdeti ABI-ja:

- `R0..R3`: argumentumok és expression temporaries;
- `R0`: skalár return value;
- `R4..R5`: callee-saved;
- `R6`: egységes stack/frame/control pointer; `PUSH/POP`, `CALL/RET` és IRQ ugyanazt használja;
- `R7`: general/address temporary;
- nincs külön rejtett hardware `SP`.

A compiler ABI implementált; további finomítás csak akkor indokolt, ha statikusan igazolhatóan csökkenti a kód- vagy végrehajtási költséget az ISA elvének torzítása nélkül.

## 18. Cycle-model javaslat

A közös SVM ciklusmodell megtartandó:

- minden fetch byte: 1 ciklus;
- byte data access: +1;
- word data access: +2;
- egyszerű ALU belső pluszköltség nélkül;
- `MUL`: +16 belső ciklus;
- `DIVU/MODU`: +16 belső ciklus (kezdeti összehasonlítási modell);
- `MULQ15`: +17 belső ciklus;
- nincs pipeline branch penalty.

Ennek következtében egy 16 bites normál ALU utasítás 2 ciklus fetch-költségű, az `LD8/ST8` 3, az `LD16/ST16` 4 ciklus alapból.

## 19. Meglévő programok kézi lowering-ellenőrzése

A v0.3 specifikációt a jelenlegi SVM-C példák fő műveleti mintáira ellenőriztük.

### 19.1. Lineáris byte copy

A strict load/store jelleg miatt a pointerléptetés explicit:

```asm
loop:
    LD8  R3,[R0+0]
    ST8  [R1+0],R3
    ADDI R0,1
    ADDI R1,1
    ADDI R2,-1
    BNZ  loop
```

Ez szándékosan drágább, mint az auto-incrementes Register vagy Memory-to-Memory forma; éppen ezt a költséget akarjuk mérni.

### 19.2. Framebuffer store

```asm
    VST8 [R4+0],R2
    ADDI R4,1
```

Nincs szükség új VRAM-modellre vagy MMIO-ra.

### 19.3. FFT Q15

A reprezentatív példakódhoz szükséges `MULQ15`, változó shift, tömbcímzés, összeadás/kivonás, valamint a futásidejű `16 / len` a major `C` `DIVU` műveletével természetesen lowerelhető. A v0.1-ben éppen a `DIVU/MODU` hiánya volt funkcionális rés.

### 19.4. Kis konstansok

A gyakori `0..63` konstansokat a `LDI6` egyetlen 16 bites utasítással állítja elő. A `SUBI` nem kap külön rövid 6 bites formát, viszont a hosszú-immediate családban saját dekódot kap a helyes carry/no-borrow szemantika miatt.

## 20. Várható összehasonlítási érték

Ez az ISA különösen az alábbiakat teszi mérhetővé:

- háromoperandusos kód vs. kétoperandusos Register CPU;
- fix word-kódolás vs. változó hosszúságú, kódsűrűségre optimalizált ISA;
- explicit címképzés és load/store mennyisége;
- regiszterallokáció minőségének hatása;
- egyszerű decoderért cserébe fizetett kódméret és fetch-költség.

## 21. Nem cél

A v0.3 nem tartalmaz:

- cache-t vagy pipeline-t;
- privilege módot;
- memory-mapped VRAM-ot;
- auto-increment load/store-t;
- több tucat általános regisztert;
- lebegőpontos utasításokat;
- SIMD-t.

Ezek hozzáadása rontaná a jelenlegi ár–érték összehasonlítás tisztaságát.

## 22. Executable azonosító

Normatív executable magic: `SVL\x01` (SVM Load/Store, ISA/executable v1).

A v0.3 felülvizsgálat során a `SEXT8` kikerült. Az SVM-C azóta támogat `i8` típust, de a sign-extension olcsón előállítható 16 bites integer műveletekből, ezért külön hardveres primitív továbbra sem indokolt. A fenntartott unary kódpont csak akkor használható fel erre, ha későbbi compiler-tapasztalat szerint a művelet gyakori, természetes backend-primitívvé válik.

A magic az assembler/runtime implementációval együtt lefoglalásra került.


## 23. Többwordös integer segédletek

A major `C` arithmetic extension további funkciói: `f=3 ADC`, `f=4 SBC`, `f=5 MULHU`, `f=6 RCR1`. Az `ADC/SBC` a C flaget viszi tovább; `MULHU` unsigned 16x16 szorzat felső 16 bitje. A `SHL1/SHR1` small-immediate formák a kieső bitet C-be írják.
