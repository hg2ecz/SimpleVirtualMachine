# SVM Memory-to-Memory CPU – ISA specifikáció (implementált v1)

## 1. Cél

A Memory-to-Memory CPU a Load/Store gép ellenpontja. A fő tervezési elv, hogy az adatfeldolgozó utasítások **közvetlenül memóriaoperandusokon** dolgozhatnak, ezért sok explicit load/store és adatregiszter-mozgatás eltűnik.

A cél egy költségorientált, ortogonális, de még implementálható CISC-jellegű összehasonlító gép. Nem cél x86, VAX vagy 68k másolása.

## 2. Közös platformprofil

A gép ugyanazt a platformot használja, mint a többi SVM CPU:

- 16 bites little-endian adatok;
- 64 KiB CPU-címtér;
- közös MMIO;
- nincs System ROM;
- külön 16 KiB VRAM;
- közös karaktergenerátor, konzol, timer és IRQ;
- instruction fetch csak CPU-memóriából.

## 3. Tervezési alapelvek

1. **Nincs általános célú adatregiszter-készlet.**
2. Az ALU tipikus alakja: `OP dst,src`, ahol `dst` és `src` memóriaoperandus.
3. Négy 16 bites **address register** (`A0..A3`) engedélyezett; ezek csak effektív címek képzésére és pointerléptetésre szolgálnak.
4. A gyakori zero-page és pointeres operandusok egyetlen operandus-descriptor bájttal kódolhatók.
5. Abszolút 16 bites cím és immediate érték hosszabb descriptor extensiont használ.
6. Az operandus-descriptor rendszer egységes a legtöbb ALU- és move-utasításra.
7. CPU-memória és VRAM külön címtér marad; általános ALU nem dolgozik VRAM-on.
8. A C fordító használhat compiler-owned zero-page scratch slotokat; ennek memóriaforgalma az architektúra valódi költsége.

## 4. Programmer-visible állapot

- `A0..A3`: négy 16 bites address register.
- `PC`: 16 bit.
- `Z`, `N`, `C` flagek.
- global interrupt enable.

Nincs `A`, `W`, `R0..Rn` jellegű általános adatregiszter.

Az address registerek adattartalma nem használható közvetlen ALU-operandusként; erre csak külön `LEA/MOVA/ADDA` address-control utasítások vannak. Ez tartja tisztán a memory-to-memory modellt.

## 5. Alap utasításformátum

A normál kétoperandusos utasítás:

`opcode8  dst_descriptor  [dst_extension]  src_descriptor  [src_extension]`

Az opcode meghatározza az adatszélességet és a műveletet. A descriptor határozza meg az operandus címzési módját.

A legrövidebb kétoperandusos művelet 3 bájt: 1 opcode + 1 dst descriptor + 1 src descriptor.

## 6. Operandus descriptor

### 6.1. CPU-memória descriptorok

| Descriptor | Mód | Extension | Effektív cím |
|---|---|---|---|
| `00..7F` | zero-page direct | nincs | `0x00dd` |
| `80..83` | `[A0]..[A3]` | nincs | `A#` |
| `84..87` | `[A0+]..[A3+]` | nincs | hozzáférés után +méret |
| `88..8B` | `[-A0]..[-A3]` | nincs | hozzáférés előtt -méret |
| `8C..8F` | `[A0+off8]..` | +1 signed byte | `A# + sign_extend(off8)` |
| `F0` | absolute16 | +2 byte | `addr16` |
| `F1` | immediate16 | +2 byte | csak source, érték és nem cím |
| `F2` | immediate8 | +1 byte | source; byte műveletnél 8 bit, word műveletnél zero-extended 8 bit |
| `F3..FF` | reserved | - | trap |

Az implementált ABI-ban `A3` az egységes stack pointer: resetkor `A3=0xFF00`, lefelé nő a `0xFB00..0xFEFF` RAM-tartományban, és ugyanazt a vermet használja a compiler temporary stack, a `CALL/RET` és az IRQ mentés. Emiatt `A3` C-kódgenerálásban nem szabad általános hosszú életű pointerként kezelni.

A `00..7F` direct descriptor a `0x0000..0x007F` tartományt fedi. Ez szándékos „hot direct memory” terület. A teljes zero page a `F0` absolute formával továbbra is elérhető.

Az `F2` word műveletben zero-extended `0..255` immediatet jelent. Ez minimális decoder-költséggel lényegesen tömörebbé teszi a C-ben gyakori kis konstansokat (`0`, `1`, ciklushatárok, kis maszkok). `F1` továbbra is a teljes 16 bites immediate forma.

### 6.2. Méretfüggő pointerléptetés

- byte műveletnél post/pre lépés: 1;
- word műveletnél: 2.

A pointer update nem módosítja a flageket.

## 7. Kétoperandusos CPU-memória műveletek

A `dst` minden write-back műveletnél írható CPU-memóriaoperandus kell legyen; immediate destination tiltott.

### 7.1. Byte műveletek

| Opcode | Mnemonic | Szemantika |
|---:|---|---|
| `10` | `MOV8 dst,src` | `dst8 = src8` |
| `11` | `ADD8 dst,src` | `dst8 += src8` |
| `12` | `SUB8 dst,src` | `dst8 -= src8` |
| `13` | `AND8 dst,src` | bitenkénti AND |
| `14` | `OR8 dst,src` | OR |
| `15` | `XOR8 dst,src` | XOR |
| `16` | `CMP8 dst,src` | flagek `dst8-src8` alapján |

### 7.2. Word műveletek

| Opcode | Mnemonic | Szemantika |
|---:|---|---|
| `20` | `MOV16 dst,src` | `dst16 = src16` |
| `21` | `ADD16 dst,src` | `dst16 += src16` |
| `22` | `SUB16 dst,src` | `dst16 -= src16` |
| `23` | `AND16 dst,src` | AND |
| `24` | `OR16 dst,src` | OR |
| `25` | `XOR16 dst,src` | XOR |
| `26` | `CMP16 dst,src` | compare, nincs write-back |
| `27` | `MUL16 dst,src` | alsó 16 bit |
| `28` | `DIV16 dst,src` | unsigned, zero divisor trap |
| `29` | `MOD16 dst,src` | unsigned |
| `2A` | `SHL16 dst,src` | `src & 15` |
| `2B` | `SHR16 dst,src` | logikai |
| `2C` | `MULQ15 dst,src` | signed Q15 |

A `CMP` destinationje olvasható, de nem módosul.

## 8. Unary memóriautasítások

Formátum: `opcode8 dst_descriptor [extension]`.

| Opcode | Mnemonic |
|---:|---|
| `30` | `INC8 dst` |
| `31` | `DEC8 dst` |
| `32` | `NOT8 dst` |
| `33` | `NEG8 dst` |
| `38` | `INC16 dst` |
| `39` | `DEC16 dst` |
| `3A` | `NOT16 dst` |
| `3B` | `NEG16 dst` |
| `3C` | `ASR1 dst` |

Ezek közvetlen read-modify-write memóriautasítások; ez a gép egyik lényegi vizsgálati pontja.

## 9. Address-register műveletek

Ezek nem általános adat-ALU műveletek, hanem kizárólag címképzésre szolgálnak.

| Opcode/form | Mnemonic | Szemantika |
|---|---|---|
| `40+r` + `imm16` | `LEA Ar,addr16` | `Ar=addr16` |
| `44+r` + `simm8` | `ADDA Ar,simm8` | `Ar += simm8` |
| `48+r` + descriptor | `MOVA Ar,mem16` | 16 bites memóriaérték -> címregiszter |
| `4C+r` + descriptor | `STORA mem16,Ar` | címregiszter -> 16 bites memória |

`r=0..3`. Az address műveletek flaget nem módosítanak.

A `MOVA/STORA` szükséges pointerek és C pointerváltozók kezeléséhez, de nem teszi az `A#` regisztereket általános adatregiszterré.

## 10. Control flow

### 10.1. Rövid relatív

| Opcode | Mnemonic | Formátum |
|---:|---|---|
| `50` | `BRA rel8` | 2 byte |
| `51` | `BZ rel8` | 2 byte |
| `52` | `BNZ rel8` | 2 byte |
| `53` | `BC rel8` | 2 byte |
| `54` | `BNC rel8` | 2 byte |
| `55` | `BN rel8` | 2 byte |
| `56` | `BNN rel8` | 2 byte |
| `57` | `CALLR rel8` | 2 byte |

A displacement a következő utasítás címéhez képest signed byte offset.

### 10.2. Hosszú abszolút

| Opcode | Mnemonic |
|---:|---|
| `58` | `JMP addr16` |
| `59` | `JZ addr16` |
| `5A` | `JNZ addr16` |
| `5B` | `JC addr16` |
| `5C` | `JNC addr16` |
| `5D` | `JN addr16` |
| `5E` | `JNN addr16` |
| `5F` | `CALL addr16` |

A v1 runtime mind a rövid relatív, mind a hosszú abszolút control-flow formát dekódolja. A jelenlegi assembler az explicit rövid displacement formát és a stabil hosszú labeles formát támogatja; automatikus branch relaxation későbbi optimalizáció.

## 11. Fixed/system utasítások

| Opcode | Mnemonic |
|---:|---|
| `00` | `NOP` |
| `01` | `HALT` |
| `02` | `RET` |
| `03` | `EI` |
| `04` | `DI` |
| `05` | `IRET` |
| `06..0F` | reserved |

A vezérlésátadás nem használ külön rejtett control-stack pointert. A `CALL/RET` és az interrupt mechanizmus az ABI szerinti `A3` stack pointert használja ugyanazon RAM-veremben, mint a compiler expression temporary-k. Ez megszünteti a két, egymást nem ismerő verem ütközésének lehetőségét.

## 12. VRAM műveletek

Az általános `MOV/ALU` descriptorok kizárólag CPU-memóriát címeznek. A VRAM elkülönítése normatív.

A video mozgatások külön opcode-családot használnak. A video oldali cím **csak** `A0..A3` pointeres móddal vagy abszolút címmel adható meg; a direct zero-page descriptor nem jelent video zero page-et.

| Opcode | Mnemonic | Szemantika |
|---:|---|---|
| `60` | `VLD8 dst_cpu,src_video` | VRAM -> CPU memória |
| `61` | `VLD16 dst_cpu,src_video` | VRAM -> CPU memória |
| `62` | `VST8 dst_video,src_cpu` | CPU-forrás descriptor -> VRAM |
| `63` | `VST16 dst_video,src_cpu` | CPU-forrás descriptor -> VRAM |
| `64..6F` | reserved | trap |

A `src_video/dst_video` descriptorok engedélyezett alakjai: `80..8F` pointeres módok és `F0` absolute16. A pointer-update ugyanúgy méretfüggő.

A `VST8/VST16` forrása a normál CPU-source descriptor szabályait használja, ezért `F1`/`F2` immediate descriptor is megengedett. Így a framebuffer konstanssal tölthető scratch memória nélkül, miközben nincs szükség külön `VSTI8/VSTI16` opcode-ra. A v0.3-ban ez a két redundáns opcode kikerült.

## 13. Flagek

- `Z`: az eredmény nulla.
- `N`: word műveletnél bit15, byte műveletnél bit7.
- `C` addition: carry out.
- `C` subtraction/compare: 1 = nincs borrow.

`MOV`, address-control, video move, branch/call/return nem módosít flaget.

## 14. Read-modify-write és MMIO

A memory-to-memory ALU utasítás `dst` operandusa logikailag:

1. egyszer kiolvassa a destinationt;
2. kiolvassa a source-ot;
3. végrehajtja az ALU-t;
4. egyszer visszaírja a destinationt.

MMIO destination esetén ez a sorrend normatív és megfigyelhető lehet. Emiatt compiler/assembler **nem használhat read-modify-write ALU-t write-only vagy side-effectes MMIO regiszteren**, csak `MOV8/MOV16` jellegű írást.

Ha `src` és `dst` ugyanarra a side-effectes MMIO címre mutat, az eredmény platformfüggő hatás helyett programozási hibának tekintendő; a dokumentáció ezt kerülendőnek minősíti.

## 15. Operandus-kiértékelési sorrend

A determinisztikus emuláció érdekében:

1. a destination effektív cím képződik;
2. a destination pre-decrement végrehajtódik, ha van;
3. a source effektív cím képződik;
4. a source pre-decrement végrehajtódik;
5. destination read;
6. source read;
7. ALU/write;
8. destination post-increment;
9. source post-increment.

`MOV` esetén destination read természetesen nincs. `CMP` esetén write nincs.

Ha ugyanaz az `A#` szerepel mindkét operandusban update móddal, a fenti sorrend kötelező; az assembler warningot adhat, de az encoding érvényes.

## 16. C compiler modell

A javasolt backend-stratégia:

- `A0`: software stack/frame pointer;
- `A1`: elsődleges source pointer;
- `A2`: elsődleges destination pointer;
- `A3`: temporary/address expression pointer;
- `0x00..0x1F`: compiler-owned hot scratch/data területből csak a linker által lefoglalt rész;
- skalár expression eredmények lehetőség szerint közvetlenül a végső memóriacélban készüljenek;
- pointeres lineáris ciklusoknál `[A+]` módot kell preferálni.

A compiler-owned zero-page tartomány pontos mérete ABI-kérdés, nem ISA-követelmény. A user program és runtime számára a linkernek ütközésmentesen kell lefoglalnia.


## 17. C ABI a jelenlegi SVM-C modellhez

A jelenlegi SVM-C statikusan allokálja a globálisokat, paramétereket és lokális változókat; rekurzió/reentrancia nincs. A v1 Memory-to-Memory backend ezt a modellt követi.

A register-backend közös expression loweringjában szereplő virtuális `R0..R7` értékek **nem hardverregiszterek**: a backend a `0x0000..0x000F` compiler-owned hot scratch memóriahelyekre képezi őket (`R0 -> 0x0000`, `R1 -> 0x0002`, ...). Emiatt ennél a targetnél a felhasználói/static C objektumok kiosztása `0x0020`-tól indul.

- a függvényargumentumok a compiler scratch értékeken keresztül kerülnek a callee statikus paraméterhelyeire;
- a skalár return value a közös `R0` scratch wordben (`0x0000`) tér vissza;
- ez a nyelv jelenlegi nem-rekurzív modelljében biztonságos;
- `A0..A3` kizárólag effektív címekhez használható, nem rejtett adatregiszter;
- `A3` az egységes stack pointer, resetkor `A3=0xFF00`, lefelé nő a `0xFB00..0xFEFF` tartományban, és a compiler expression temporaries mellett `CALL/RET/IRQ` is ugyanezt használja.

Ez az ABI tudatosan láthatóvá teszi a regiszter nélküli adatmodell memóriaforgalmi költségét, miközben nem tesz általános adatregisztert a CPU-ba.

## 18. Cycle-model javaslat

A közös SVM modell szerint:

- minden instruction/descriptor/extension fetch byte +1 ciklus;
- minden byte data read/write +1;
- minden word read/write +2;
- egyszerű ALU belső pluszköltség nélkül;
- multiply/divide a meglévő CPU-kkal összevethető belső költséget kap.

Példa: `ADD16 0x10,0x11` direct-direct encoding 3 fetch byte + 2 word read + 1 word write = `3 + 2 + 2 + 2 = 9` ciklus az alapmodellben.

Ez szándékosan láthatóvá teszi a kisebb instruction count és a nagyobb memóriaforgalom közötti kompromisszumot.

## 19. Kódsűrűségi példák

### 19.1. Két hot word összeadása

```asm
ADD16 0x10, 0x12
```

3 bájt, egy utasítás.

### 19.2. Pointeres másolási ciklus törzse

```asm
MOV8 [A1+], [A0+]
```

3 bájt; két pointerléptetés implicit.

Ez a tervezés egyik legerősebb esete és közvetlen ellenpontja a strict load/store CPU-nak.

### 19.3. Abszolút változó + konstans

```asm
ADD16 [0x1234], 7
```

`opcode + F0 + addr16 + F1 + imm16` = 7 bájt. A CISC rugalmasság nem feltétlenül jelent mindig rövid kódot; ennek költsége a generált kódban közvetlenül vizsgálható.

## 20. Meglévő programok kézi lowering-ellenőrzése

### 20.1. Lineáris memória-copy

```asm
loop:
    MOV8  [A1+],[A0+]
    DEC16 count
    BNZ   loop
```

A két pointer implicit léptetése ennek az ISA-nak a szándékos erőssége.

### 20.2. Framebuffer fill

Konstans kitöltésnél a `VST8 [A0+],imm8` az immediate source descriptor miatt elkerüli a mesterséges scratch-memory forgalmat. Változó pixelértéknél ugyanaz a `VST8 [A0+],src_cpu` forma használható memóriaforrás descriptorral.

### 20.3. FFT Q15

A tömbindexek és expression temporaries memóriahelyeken élnek; `A#` regiszterekkel képezzük a dinamikus címeket. A `MULQ15`, `DIV16`, `SHL16/SHR16`, memória–memória `ADD/SUB` és `ASR1` együtt lefedi a meglévő reprezentatív programok műveleteit. Az ebből származó scratch-memory forgalom nem hiba, hanem a regiszter nélküli adatmodell mérendő költsége.

### 20.4. Kis konstansok

Word műveletekben az `F2` zero-extended imm8 descriptor teszi tömörré például az `ADD16 x,1`, `CMP16 x,16` és hasonló gyakori eseteket. Ez nem új operandusmód, csak a már meglévő immediate descriptor jobb kihasználása.

## 21. Várható összehasonlítási érték

A gép mérhetővé teszi:

- instruction count csökkenése vs. data-memory forgalom;
- explicit register allocator hiányának előnye/hátránya;
- komplex operandusdekódolás ára;
- auto-increment pointeres műveletek kódsűrűsége;
- zero-page compiler scratch jelentősége;
- memory-to-memory forma C expression tree-kre gyakorolt hatása;
- fetch bájtok és adat-hozzáférések eltérő aránya.

## 22. Nem cél

A v0.3 nem tartalmaz:

- tetszőleges háromoperandusos memory-to-memory ALU-t;
- index*scale címzést;
- segment/bank rendszert;
- string-repeat mikroprogramokat;
- memória–VRAM ALU-t;
- általános data registert;
- lebegőpontos vagy SIMD egységet.

Ezek jelentősen növelnék a decoder és compiler komplexitását anélkül, hogy az alapelv összehasonlításához szükségesek lennének.

## 23. Executable azonosító

Normatív executable magic: `SVC\x01` (SVM CISC / memory-to-memory, ISA/executable v1).

Az implementáció kezdetén kell véglegesíteni.


## Integer segédletek

A 16 bites memory-to-memory műveletekhez `ADC16`=`2D`, `SBC16`=`2E`, `MULHU16`=`2F` tartozik. A unary `RCR1`=`3D`, `SHL1`=`3E`, `SHR1`=`3F`; az egybites shiftek a kieső bitet C-be írják. Hardveres floating point nincs.
