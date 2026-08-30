# Architektúratervezési indoklás és ár–érték politika

## Cél

A projekt célja nem egyetlen „legjobb” CPU kiválasztása, hanem lényegesen eltérő egyszerű operandus- és végrehajtásszervezési modellek összehasonlítása azonos 16 bites platformon, azonos perifériákkal, assemblerrel, runtime-mal és SVM-C nyelvi környezettel.

A tervezés fő szabálya: **csak az a hardveres vagy ISA-bővítés maradjon meg, amely a kódméretet, a memóriaforgalmat, a fordító egyszerűségét vagy a kézi assembly használhatóságát érdemben javítja a hozzáadott állapot/dekóderköltséghez képest.**

## A kilenc architektúra szerepe

| CPU | Elsődleges szervezési elv | Miért értékes összehasonlítási pont? |
|---|---|---|
| Accumulator | implicit központi adatoperandus | nagyon kis állapot és egyszerű adatút |
| Stack | 0-címes operandusstack | nagy kódsűrűség, implicit operandusok |
| MemReg | working register + direct file memory | kis direct tér és pointeres adatjárás |
| Register | kétoperandusos regisztergép | klasszikus általános GPR modell |
| Load/Store | háromoperandusos strict RISC | explicit memóriaforgalom, szabályos adatút |
| Register-Memory | GPR + közvetlen memória source | a strict RISC és CISC közti költségpont |
| Memory-to-Memory | közvetlen memóriaoperandusok | minimális programmer-visible regiszterállapot |
| Belt16 | implicit eredményszalag | célregiszter nélküli eredménykezelés |
| TTA16 | MOVE-központú transport-triggered végrehajtás | funkcionális egység portokra épülő alternatív modell |

A család így lefedi a fő egyszerű klasszikus operandusmodelleket, valamint két nem klasszikus végrehajtásszervezési irányt. További ISA csak akkor indokolt, ha valóban új mérési kérdést ad.

## Miért nem része a családnak néhány további irány?

- **OISC/SUBLEQ:** extrém egyszerű dekóder, de túl nagy kód- és memóriaforgalmi büntetés; kevés új gyakorlati információt ad.
- **VLIW:** széles fetch és statikus ütemezés más problématerületet vizsgálna, és megtörné a kis, összehasonlítható CPU-k közös keretét.
- **külön Harvard CPU-változat:** ez elsősorban memória-architektúra, nem új operandusmodell.
- **hardveres floating point:** túl nagy állapot- és adatút-költség a projekt céljához; az f16/f32 soft-float könyvtár tudatosan szoftveres.
- **32/64 bites általános ALU:** a 16 bites architektúrák összehasonlíthatóságát rontaná. A többwordös műveletek könyvtári loweringgel készülnek.

## Utasítás-megtartási politika

Egy utasítás értékét három azonos súlyú szempont szerint kell nézni:

1. **compiler-essential** – a C backend, wide-int vagy soft-float lowering közvetlenül profitál belőle;
2. **generally useful** – C-ben és kézi assemblyben is gyakori, kis költségű primitív;
3. **assembly-oriented** – főleg kézzel írt, az ISA természetes stílusát követő assembly miatt értékes.

Az „assembly-oriented” nem deprecated és nem opcionális státusz.

Egy utasítás elhagyása akkor indokolt, ha legalább kettő teljesül:

- a C backend és a reprezentatív assembly példák sem használják;
- rövid, azonos szemantikájú sorozattal kiváltható jelentős spill/memóriaforgalom nélkül;
- eltávolítása tényleges hardverállapotot, dekóderkomplexitást vagy kritikus adatutat csökkent, nem csak opcode-helyet szabadít fel.

## Megtartott kis költségű integer segítségek

A közös ár–érték felülvizsgálat alapján megtartott, hardverben olcsó és szoftverben hasznos primitívek:

- `ADC`, `SBC` – többwordös összeadás/kivonás carry-láncaihoz;
- `MULHU`, illetve Stack célon `UMUL (a b -- lo hi)` – 16×16→32 eredményhez;
- `RCR1` – carryn keresztüli többwordös jobbra toláshoz;
- `SHL1`, `SHR1` carry-outtal;
- `MULQ15` – fixpontos DSP/FFT feladatokhoz;
- `DIV`, `MOD` – a C nyelv és a könyvtárak közvetlen igénye miatt;
- rövid branch/call, zero-page/direct és természetes post-increment/pre-decrement formák ott, ahol ténylegesen rövidebb kódot adnak.

Szándékosan nincs külön CLZ, általános 32 bites ALU vagy hardveres float.

## Fontos redundancia-döntések

### `SUBI` nem általánosan helyettesíthető `ADDI -imm` formával

A numerikus 16 bites eredmény azonos lehet, de a carry/no-borrow flag szemantikája nem feltétlenül az. Emiatt ahol a flag része az ISA szerződésének, a natív `SUBI` megmarad.

### `SEXT8`

A sign extension szoftver/lowering oldalon olcsón előállítható, ezért külön opcode általában nem fizeti vissza a dekóderhelyét.

### Stack assembly-orientált család

A következők tudatosan megmaradnak a Forth-szerű és kézi assembly használhatóság miatt:

- `NIP`, `TUCK`, `2DUP`, `2DROP`;
- `PICK`, `ROLL`;
- `DO`, `?DO`, `I`, `J`, `LOOP`, `+LOOP`, `LEAVE`, `UNLOOP`.

## Stack mikroarchitektúra: TOS+NOS lazy cache

A Stack CPU ár–érték szempontból két felső stackelemet tart regiszterben:

```text
TOS  = stack[0]
NOS  = stack[1], ha érvényes
RAM  = mélyebb elemek
```

A második elem csak akkor töltődik vissza RAM-ból, amikor egy utasítás ténylegesen igényli. Bináris ALU-művelet után nincs automatikus refill. Ez az ISA módosítása nélkül csökkenti a stack-RAM forgalmat.

A választás oka:

- egyetlen további 16 bites regiszter és kevés vezérlési állapot;
- a legtöbb kétoperandusos kifejezés regiszterben maradhat;
- `SWAP`, `DUP`, `DROP`, `NIP` és több gyakori stackművelet olcsóbb lesz;
- nagyobb, 4–8 elemű rejtett register-stack cache már aránytalanul növelné a mux- és vezérlési költséget.

A Stack továbbra is valódi stack architektúra; a cache mikroarchitekturális optimalizáció, nem programmer-visible register file.

## Közös platformdöntések

- A CPU-címtér `0x0000..0xFEFF` tartománya összefüggő RAM, a `0xFF00..0xFFFF` tartomány MMIO.
- A VRAM külön video-címtér, így nem töri meg a CPU program/adat RAM folytonosságát.
- A karakter-ROM a videoeszköz belső erőforrása, nem CPU-címezhető.
- A perifériamodell közös mind a kilenc CPU-n, hogy az ISA-k összevetése tiszta maradjon.

## Mérési szemlélet

Az architektúrákat ugyanazon programokon érdemes összevetni legalább:

- executable kódméret;
- retired instruction count;
- instruction-fetch költség;
- CPU RAM és VRAM hozzáférések;
- VM ciklusszám;
- compiler spill/scratch forgalom;
- programmer-visible állapot és dekóder/adatút egyszerűsége.

A ciklusszám absztrakt VM-ciklusmodell, nem közvetlen MHz vagy falióra. A hardveres ár–érték értékelésnél a kritikus út és a reálisan elérhető Fmax is külön szempont.

## Kapcsolódó normatív dokumentumok

- [ISA képességmátrix](ISA_CAPABILITY_MATRIX_HU.md)
- [ISA referencia](ISA_REFERENCE_HU.md)
- [Platform](PLATFORM_HU.md)
- [MMIO referencia](MMIO_REFERENCE_HU.md)
- [Implementációs állapot](IMPLEMENTATION_STATUS_HU.md)
