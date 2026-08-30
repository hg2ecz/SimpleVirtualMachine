# Akkumulátoros CPU utasításreferencia

A CPU állapota szándékosan kicsi: 16 bites `A`, 16 bites `X`, olcsó 16 bites `Y` címregiszter, 16 bites `PC`, 16 bites `SP`, valamint `Z/N/C` flag-ek. A többbájtos operandusok little-endian kódolásúak.

| Hex | Utasítás | Bájt | Jelentés |
|---:|---|---:|---|
| `00` | `NOP` | 1 | Nincs művelet |
| `01` | `HALT` | 1 | CPU leállítása |
| `02` | `RET` | 1 | Visszatérési cím pop `PC`-be |
| `03` | `TAX` | 1 | `X=A` |
| `04` | `TXA` | 1 | `A=X` |
| `05` | `PUSHA` | 1 | `A` push |
| `06` | `POPA` | 1 | `A` pop |
| `07` | `PUSHX` | 1 | `X` push |
| `08` | `POPX` | 1 | `X` pop |
| `09` | `INC` | 1 | `A=A+1` |
| `0A` | `DEC` | 1 | `A=A-1` |
| `0B` | `NEG` | 1 | `A=0-A` |
| `0C` | `NOT` | 1 | `A=~A` |
| `0D` | `SHL1` | 1 | `A<<=1` |
| `0E` | `SHR1` | 1 | logikai `A>>=1` |
| `0F` | `INX` | 1 | `X=X+1` |
| `10` | `DEX` | 1 | `X=X-1` |
| `11` | `ADDX` | 1 | `A=A+X` |
| `12` | `SUBX` | 1 | `A=A-X` |
| `13` | `MULX` | 1 | `A=A*X` |
| `14` | `DIVX` | 1 | unsigned `A=A/X` |
| `15` | `MODX` | 1 | unsigned `A=A%X` |
| `16` | `ANDX` | 1 | `A=A&X` |
| `17` | `ORX` | 1 | `A=A|X` |
| `18` | `XORX` | 1 | `A=A^X` |
| `19` | `SHLX` | 1 | `A <<= (X&15)` |
| `1A` | `SHRX` | 1 | `A >>= (X&15)` |
| `1B` | `CMPX` | 1 | flag-ek `A-X` alapján, `A` nem változik |
| `1C` | `LDA8 [X]` | 1 | X címen lévő byte zero-extend `A`-ba |
| `1D` | `LDA16 [X]` | 1 | X címen lévő word betöltése `A`-ba |
| `1E` | `STA8 [X]` | 1 | `A` alsó byte-jának tárolása X címre |
| `1F` | `STA16 [X]` | 1 | `A` tárolása X címre |
| `20` | `LDA8 [X+]` | 1 | byte load, majd `X+=1` |
| `21` | `LDA16 [X+]` | 1 | word load, majd `X+=2` |
| `22` | `STA8 [X+]` | 1 | byte store, majd `X+=1` |
| `23` | `STA16 [X+]` | 1 | word store, majd `X+=2` |
| `40` | `LDAI imm16` | 3 | `A=imm16` |
| `41` | `LDXI imm16` | 3 | `X=imm16` |
| `42` | `ADDI imm16` | 3 | `A+=imm16` |
| `43` | `SUBI imm16` | 3 | `A-=imm16` |
| `44` | `CMPI imm16` | 3 | flag-ek `A-imm16` alapján |
| `45` | `ANDI imm16` | 3 | `A&=imm16` |
| `46` | `ORI imm16` | 3 | `A|=imm16` |
| `47` | `XORI imm16` | 3 | `A^=imm16` |
| `50` | `LDA8 addr16` | 3 | abszolút byte load |
| `51` | `LDA16 addr16` | 3 | abszolút word load |
| `52` | `STA8 addr16` | 3 | abszolút byte store |
| `53` | `STA16 addr16` | 3 | abszolút word store |
| `60` | `JMP addr16` | 3 | feltétel nélküli ugrás |
| `61` | `CALL addr16` | 3 | return PC push és ugrás |
| `62` | `JZ addr16` | 3 | ugrás, ha `Z=1` |
| `63` | `JNZ addr16` | 3 | ugrás, ha `Z=0` |
| `64` | `JC addr16` | 3 | ugrás, ha `C=1` |
| `65` | `JNC addr16` | 3 | ugrás, ha `C=0` |
| `66` | `JN addr16` | 3 | ugrás, ha `N=1` |
| `67` | `JNN addr16` | 3 | ugrás, ha `N=0` |

A `3A..3F`, `49..4F`, `54..5F` és `70..FF` opcode-tartományok fenntartottak. A `30..39` tartomány a zero-page, interrupt/DSP és videó-kiterjesztéseké. A szabad opcode-helyet tudatosan nem tekintjük önmagában új hardverfunkció indokának.

Az aritmetikai műveletek frissítik `Z/N`-t és ahol értelmes, `C`-t. A `CMPX/CMPI` a flag-eket frissíti `A` módosítása nélkül.

## Kódolás, hossz és végrehajtási idő – gyors referencia

A ciklusidők a `../../../svm_rt/docs/CYCLE_MODEL.md` modellt követik. Az utasításfetch byte-onként számít, a byte adat-hozzáférés +1 ciklus, a word adat-hozzáférés +2, és az explicit iteratív aritmetika hozzáadja a belső költségét. Nincs taken-branch büntetés.

| Assembly forma | Hex kódolás | Bájt | Ciklus | Megjegyzés |
|---|---|---:|---:|---|
| `NOP`, `HALT`, regiszter transfer/inc/dec/unary, `EI`, `DI`, `ASR1` | megfelelő egybájtos opcode | 1 | 1 | belső regiszterművelet |
| `RET`, `PUSHA/POPA`, `PUSHX/POPX` | `02`, `05..08` | 1 | 3 | egy 16 bites stack-hozzáférés |
| `IRET` | `36` | 1 | 5 | két 16 bites stack read |
| `ADDX/SUBX/ANDX/ORX/XORX/CMPX` | `11,12,16..18,1B` | 1 | 1 | regiszteres ALU |
| `MULX`, `DIVX`, `MODX` | `13..15` | 1 | 17 | 1 fetch + 16 belső |
| `SHLX`, `SHRX` | `19`, `1A` | 1 | 2 | +1 változó-shift belső ciklus |
| X/Y byte load/store | `1C,1E,20,22,28,2A,2C,2E` | 1 | 2 | +1 byte adat-hozzáférés |
| X/Y word load/store | `1D,1F,21,23,29,2B,2D,2F` | 1 | 3 | +2 word adat-hozzáférés |
| zero-page byte load/store | `30 aa`, `32 aa` | 2 | 3 | 2 fetch + byte hozzáférés |
| zero-page word load/store | `31 aa`, `33 aa` | 2 | 4 | 2 fetch + word hozzáférés |
| `MULQ15X` | `38` | 1 | 18 | 1 fetch + 17 belső |
| videó byte load/store | `39 ss` | 2 | 3 | külön videó-adattér |
| videó word load/store | `39 ss` | 2 | 4 | külön videó-adattér |
| `LDAI/LDXI/LDYI/ADDI/SUBI/CMPI/ANDI/ORI/XORI imm16` | `40..48 lo hi` | 3 | 3 | immediate fetch |
| abszolút byte load/store | `50/52 lo hi` | 3 | 4 | +1 adat-hozzáférés |
| abszolút word load/store | `51/53 lo hi` | 3 | 5 | +2 adat-hozzáférés |
| hosszú `JMP/Jcc addr16` | `60,62..67 lo hi` | 3 | 3 | taken/not-taken azonos |
| hosszú `CALL addr16` | `61 lo hi` | 3 | 5 | +16 bites stack write |
| rövid `RJMP/RJcc rel8` | `68,6A..6F dd` | 2 | 2 | assembler automatikusan választja |
| rövid `RCALL rel8` | `69 dd` | 2 | 4 | +16 bites stack write |

A post-increment és pre-decrement címfrissítés nem ad külön ciklust a memória-hozzáférésen felül.

## Második címregiszter és kétirányú memóriabejárás

`Y` olcsó második cím-/indexregiszter, nem általános ALU-regiszter. Elsődleges célja, hogy másolás és bufferfeldolgozás közben a forrás- és célpointer egyszerre rezidens maradjon.

| Utasítás | Opcode | Jelentés |
|---|---:|---|
| `TAY` | `24` | `Y=A` |
| `TYA` | `25` | `A=Y` |
| `INY` | `26` | `Y=Y+1` |
| `DEY` | `27` | `Y=Y-1` |
| `STA8 [Y]` | `28` | `mem8[Y]=A` |
| `STA16 [Y]` | `29` | `mem16[Y]=A` |
| `STA8 [Y+]` | `2A` | store, majd `Y=Y+1` |
| `STA16 [Y+]` | `2B` | store, majd `Y=Y+2` |
| `LDA8 [-X]` | `2C` | `X=X-1; A=mem8[X]` |
| `LDA16 [-X]` | `2D` | `X=X-2; A=mem16[X]` |
| `STA8 [-Y]` | `2E` | `Y=Y-1; mem8[Y]=A` |
| `STA16 [-Y]` | `2F` | `Y=Y-2; mem16[Y]=A` |
| `LDYI imm16` | `48` | `Y=imm16` |

## Rövid vezérlésátadás és relaxation

A forrás-mnemonikák továbbra is `JMP`, `CALL`, `JZ`, `JNZ`, `JC`, `JNC`, `JN`, `JNN`. Az assembler automatikusan a kétbájtos PC-relatív kódolást választja, ha a cél a következő `PC`-hez képest signed 8 bites displacementtel elérhető; egyébként a hárombájtos abszolút formát generálja. A forrásnak ezért nem kell külön short mnemonikát használnia.

| Hex | Belső rövid forma | Bájt | Jelentés |
|---:|---|---:|---|
| `68` | `RJMP rel8` | 2 | `PC=next_pc+sign_extend(rel8)` |
| `69` | `RCALL rel8` | 2 | `next_pc` push, majd relatív call |
| `6A` | `RJZ rel8` | 2 | relatív ugrás, ha `Z=1` |
| `6B` | `RJNZ rel8` | 2 | relatív ugrás, ha `Z=0` |
| `6C` | `RJC rel8` | 2 | relatív ugrás, ha `C=1` |
| `6D` | `RJNC rel8` | 2 | relatív ugrás, ha `C=0` |
| `6E` | `RJN rel8` | 2 | relatív ugrás, ha `N=1` |
| `6F` | `RJNN rel8` | 2 | relatív ugrás, ha `N=0` |

Ez kizárólag kódsűrűségi optimalizáció; távoli célhoz a hosszú abszolút formák automatikusan megmaradnak. Az aktuális accumulator executable formátum `SVA\x06`.

## Közvetlen zero-page formák

| Hex | Mnemonik | Bájt |
|---|---|---:|
| `30` | `LDA8Z addr8` | 2 |
| `31` | `LDA16Z addr8` | 2 |
| `32` | `STA8Z addr8` | 2 |
| `33` | `STA16Z addr8` | 2 |

Ezek az abszolút címzés harmadik bájtját takarítják meg külön page-base regiszter nélkül.

## Megszakításvezérlés

| Hex | Utasítás | Bájt | Hatás |
|---:|---|---:|---|
| `34` | `EI` | 1 | globális maskable interrupt engedélyezése |
| `35` | `DI` | 1 | globális maskable interrupt tiltása |
| `36` | `IRET` | 1 | mentett státusz/control és `PC` visszaállítása |

Interrupt belépés a meglévő 1 KiB hardveres stacket használja, és törli az interrupt-enable állapotot, amíg az `IRET` vissza nem állítja.

## Integer DSP kiterjesztés

| Utasítás | Hex | Jelentés |
|---|---|---|
| `ASR1` | `37` | `A` aritmetikai jobbra shiftje |
| `MULQ15X` | `38` | `A=q15(A*X)` |

A `MULQ15` signed 16 bites operandusokat, 32 bites köztes értéket és aritmetikai `>>15` visszaskálázást használ; a `0x8000*0x8000` egyedi túlcsordulás `0x7FFF`-re telítődik.

## Külön videó-címtér kiterjesztés

A `0x39` videómemória-prefix. A második bájt választja ki az X/Y címzésű videóműveletet. Bár a kódolása két bájt, egyetlen logikai VM-utasítás.

| Mnemonik | Hex |
|---|---|
| `VLDA8 [X]` / `VLDA16 [X]` | `39 00` / `39 01` |
| `VSTA8 [X]` / `VSTA16 [X]` | `39 02` / `39 03` |
| `VLDA8 [X+]` / `VLDA16 [X+]` | `39 04` / `39 05` |
| `VSTA8 [X+]` / `VSTA16 [X+]` | `39 06` / `39 07` |
| `VSTA8 [Y]` / `VSTA16 [Y]` | `39 08` / `39 09` |
| `VSTA8 [Y+]` / `VSTA16 [Y+]` | `39 0A` / `39 0B` |
| `VLDA8 [-X]` / `VLDA16 [-X]` | `39 0C` / `39 0D` |
| `VSTA8 [-Y]` / `VSTA16 [-Y]` | `39 0E` / `39 0F` |

## Többwordös integer segédutasítások

`ADCX` (`3A`) számítása `A=A+X+C`; `SBCX` (`3B`) számítása `A=A-X-(1-C)`; `MULHUX` (`3C`) az unsigned `A*X` felső 16 bitjét adja; `RCR1` (`3D`) carry-n keresztül forgat jobbra. A `SHL1/SHR1` a kilépő bitet `C`-be írja.
