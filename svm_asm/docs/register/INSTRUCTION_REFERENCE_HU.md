# Regiszteres gép utasításreferencia

Ez a dokumentum a jelenlegi költségoptimalizált regiszteres ISA (`SVM\x09`) normatív, programozói utasításdefiníciója. Minden többbájtos 16 bites immediate és cím little-endian kódolású.

## Kódolás, hossz és végrehajtási idő – gyors referencia

Az alábbi idők a `../../../svm_rt/docs/CYCLE_MODEL.md` VM-ciklusmodelljét követik: minden fetch-elt utasításbájt 1 ciklus, egy 8 bites adat-hozzáférés 1 ciklus, egy 16 bites adat-hozzáférés 2 ciklus, az explicit többciklusos aritmetika pedig hozzáadja saját belső költségét. Nincs pipeline branch penalty. `C` = ciklus.

| Assembly forma | Hex kódolás | Bájt | C | Megjegyzés |
|---|---|---:|---:|---|
| `NOP`, `HALT`, `EI`, `DI` | `00`, `01`, `07`, `08` | 1 | 1 | csak fetch |
| `RET` | `02` | 1 | 3 | fetch + 16 bites stack read |
| `ZLOAD8 addr8` / `ZSTORE8 addr8` | `03 aa` / `05 aa` | 2 | 3 | 2 fetch + byte adat-hozzáférés |
| `ZLOAD16 addr8` / `ZSTORE16 addr8` | `04 aa` / `06 aa` | 2 | 4 | 2 fetch + word adat-hozzáférés |
| `IRET` | `09` | 1 | 5 | két 16 bites stack read |
| `ASR1 Rr` | `0A rr` | 2 | 2 | a második bájt a regiszterszám |
| `MULQ15 Rd,Rs` | `0B 00dddsss` | 2 | 19 | 2 fetch + 17 belső |
| `VLOAD8/VSTORE8 ...` | `0C ss 00dddsss` | 3 | 4 | videó byte hozzáférés |
| `VLOAD16/VSTORE16 ...` | `0C ss 00dddsss` | 3 | 5 | videó word hozzáférés |
| unáris `NOT/NEG/INC/DEC/SHL1/SHR1 Rr` | `10..3F` | 1 | 1 | regiszter az opcode-ban |
| `PUSH Rr` / `POP Rr` | `40..4F` | 1 | 3 | 16 bites hardveres stack-hozzáférés |
| compact `MOV/ADD/SUB/CMP/AND Rd,Rs` (`R0..R3`) | `50..8F`, `B0..BF` | 1 | 1 | mindkét regiszter az opcode-ban |
| compact `LOAD8` / `STORE8` | `90..AF` | 1 | 2 | fetch + byte adat-hozzáférés |
| `MOVI/ADDI/SUBI/CMPI Rr,imm16` | `C0..DF lo hi` | 3 | 3 | csak immediate fetch |
| általános `MOV/ADD/SUB/AND/OR/XOR/CMP` | `E0/E1/E2/E6/E7/E8/EB pp` | 2 | 2 | `pp=00dddsss` |
| `MUL/DIV/MOD Rd,Rs` | `E3/E4/E5 pp` | 2 | 18 | 2 fetch + 16 belső |
| `SHL/SHR Rd,Rs` | `E9/EA pp` | 2 | 3 | +1 belső változó-shift költség |
| általános `LOAD8/STORE8` | `EC/EE pp` | 2 | 3 | byte adat-hozzáférés |
| általános `LOAD16/STORE16` | `ED/EF pp` | 2 | 4 | word adat-hozzáférés |
| `JMP/Jcc addr16` | `F0,F2..F7 lo hi` | 3 | 3 | taken/not-taken költség azonos |
| `CALL addr16` | `F1 lo hi` | 3 | 5 | +16 bites return-address write |
| post-inc/pre-dec byte load/store | `F8/F9/FC/FD pp` | 2 | 3 | címfrissítés belső |
| post-inc/pre-dec word load/store | `FA/FB/FE/FF pp` | 2 | 4 | címfrissítés belső |

A videó `ss` subopcode `00..0B`, ahogy később szerepel. Érvénytelen kódolás trapet okoz még azelőtt, hogy az utasítás sikeresen retire-olna.

## CPU-állapot

- Nyolc 16 bites általános regiszter: `R0..R7`.
- `R0..R3` egyben a kiválasztott egybájtos kétregiszteres utasítások compact részhalmaza.
- 16 bites `PC`.
- 16 bites hardveres `SP`, amelyet `PUSH`, `POP`, `CALL`, `RET` használ.
- Flag-ek: `Z` (zero), `N` (negative/sign bit), `C` (carry/no-borrow).

## Kódolási jelölések

- `r` / `rrr`: 3 bites regiszterszám (`0..7`).
- `d`, `s`: destination és source regiszterszám.
- `dd`, `ss`: 2 bites compact regiszterszám (`R0..R3`).
- `imm16`: 16 bites little-endian immediate.
- `addr16`: 16 bites little-endian abszolút cím.
- Általános regiszterpár-bájt: `00dddsss`.
- Compact kétregiszteres opcode: `base | (dd << 2) | ss`.

## Fix utasítások

| Mnemonik | Hex opcode | Bájt | Definíció | Flag |
|---|---:|---:|---|---|
| `NOP` | `00` | 1 | Nincs művelet. | változatlan |
| `HALT` | `01` | 1 | CPU leállítása reset/host beavatkozásig. | változatlan |
| `RET` | `02` | 1 | 16 bites visszatérési cím pop a hardveres stackről `PC`-be. | változatlan |

`0D..0F` jelenleg érvénytelen/fenntartott. A `03..0C` opcode-ok a később leírt zero-page, interrupt/DSP és videó-kiterjesztésekhez tartoznak.

## Beágyazott regiszteres unáris utasítások

Az opcode alsó három bitje választja `R0..R7` egyikét. Mindegyik pontosan egy bájt.

| Mnemonik | Opcode-tartomány | Képlet | Definíció | Flag |
|---|---:|---|---|---|
| `NOT Rr` | `10..17` | `10 | r` | `Rr = ~Rr` | `Z,N` frissül; `C` változatlan |
| `NEG Rr` | `18..1F` | `18 | r` | `Rr = 0 - Rr` | `Z,N` frissül; `C=1`, ha az operandus nulla |
| `INC Rr` | `20..27` | `20 | r` | `Rr = Rr + 1` | `Z,N,C` frissül |
| `DEC Rr` | `28..2F` | `28 | r` | `Rr = Rr - 1` | `Z,N,C` frissül (`C` = no borrow) |
| `SHL1 Rr` | `30..37` | `30 | r` | Logikai bal shift egy bittel. | `Z,N` frissül; `C` = régi bit15 |
| `SHR1 Rr` | `38..3F` | `38 | r` | Logikai jobb shift egy bittel. | `Z,N` frissül; `C` = régi bit0 |
| `PUSH Rr` | `40..47` | `40 | r` | A 16 bites regiszterérték push a hardveres stackre. | változatlan |
| `POP Rr` | `48..4F` | `48 | r` | 16 bites érték pop a hardveres stackről `Rr`-be. | változatlan |

Példa: `INC R3` kódolása `23`.

## Compact kétregiszteres utasítások (`R0..R3`)

Ezek egybájtosak. Az opcode `base | (dd << 2) | ss`, ahol `dd` és `ss` csak `R0..R3` regisztert kódol.

| Mnemonik | Hex tartomány | Base | Definíció | Flag |
|---|---:|---:|---|---|
| `MOV Rd, Rs` | `50..5F` | `50` | `Rd = Rs` | változatlan |
| `ADD Rd, Rs` | `60..6F` | `60` | `Rd = Rd + Rs` | `Z,N,C` frissül |
| `SUB Rd, Rs` | `70..7F` | `70` | `Rd = Rd - Rs` | `Z,N,C` frissül (`C` = no borrow) |
| `CMP Rd, Rs` | `80..8F` | `80` | `Rd - Rs` csak flag-ekhez. | `Z,N,C` frissül |
| `LOAD8 Rd, [Rs]` | `90..9F` | `90` | `Rd = zero_extend(mem8[Rs])` | változatlan |
| `STORE8 [Rd], Rs` | `A0..AF` | `A0` | `mem8[Rd] = low8(Rs)` | változatlan |
| `AND Rd, Rs` | `B0..BF` | `B0` | `Rd = Rd AND Rs` | `Z,N` frissül; `C` változatlan |

Példa: `ADD R2, R1` = `60 | (2 << 2) | 1` = `69`.

Az assembler automatikusan ezeket használja, ha mindkét regiszter `R0..R3`; különben az alábbi általános formát. Az `XOR` teljesen támogatott marad az általános kétregiszteres családban; az `AND` azért kapta a compact helyet, mert a wide-integer és soft-float könyvtárakban a maszkolás lényegesen gyakoribb.

## Beágyazott regiszteres immediate utasítások

Az alsó három bit `R0..R7` egyikét választja; utána két immediate bájt következik.

| Mnemonik | Opcode-tartomány | Képlet | Bájt | Definíció | Flag |
|---|---:|---|---:|---|---|
| `MOVI Rr, imm16` | `C0..C7` | `C0 | r` | 3 | `Rr = imm16` | változatlan |
| `ADDI Rr, imm16` | `C8..CF` | `C8 | r` | 3 | `Rr = Rr + imm16` | `Z,N,C` frissül |
| `SUBI Rr, imm16` | `D0..D7` | `D0 | r` | 3 | `Rr = Rr - imm16` | `Z,N,C` frissül (`C` = no borrow) |
| `CMPI Rr, imm16` | `D8..DF` | `D8 | r` | 3 | `Rr - imm16` csak flag-ekhez. | `Z,N,C` frissül |

Assembler költségcsökkentések: `ADDI Rn,1 -> INC Rn`, `SUBI Rn,1 -> DEC Rn`, `MOV Rn,Rn -> NOP`. A `SUBI` hardverutasítás marad, mert `ADDI -imm` ugyanazt a numerikus eredményt adhatná, de a megfigyelhető carry/no-borrow flag-szemantika nem minden esetben azonos.

## Általános kétregiszteres / memóriautasítások

Ezek egy opcode-bájtot és egy `00dddsss` regiszterpár-bájtot használnak; teljes hosszuk két bájt. Mind a nyolc regiszter használható.

| Mnemonik | Hex opcode | Bájt | Pair jelentés | Definíció | Flag |
|---|---:|---:|---|---|---|
| `MOV Rd, Rs` | `E0` | 2 | `d=Rd,s=Rs` | `Rd = Rs` | változatlan |
| `ADD Rd, Rs` | `E1` | 2 | ugyanaz | `Rd = Rd + Rs` | `Z,N,C` frissül |
| `SUB Rd, Rs` | `E2` | 2 | ugyanaz | `Rd = Rd - Rs` | `Z,N,C` frissül |
| `MUL Rd, Rs` | `E3` | 2 | ugyanaz | `Rd = Rd * Rs` alsó 16 bit | `Z,N` frissül; `C=0` |
| `DIV Rd, Rs` | `E4` | 2 | ugyanaz | Unsigned `Rd = Rd / Rs`; 0-val osztás trap. | `Z,N` frissül |
| `MOD Rd, Rs` | `E5` | 2 | ugyanaz | Unsigned `Rd = Rd % Rs`; 0-val osztás trap. | `Z,N` frissül |
| `AND Rd, Rs` | `E6` | 2 | ugyanaz | `Rd = Rd AND Rs` | `Z,N` frissül |
| `OR Rd, Rs` | `E7` | 2 | ugyanaz | `Rd = Rd OR Rs` | `Z,N` frissül |
| `XOR Rd, Rs` | `E8` | 2 | ugyanaz | `Rd = Rd XOR Rs` | `Z,N` frissül |
| `SHL Rd, Rs` | `E9` | 2 | ugyanaz | Logikai bal shift `(Rs & 15)` bittel. | `Z,N` frissül |
| `SHR Rd, Rs` | `EA` | 2 | ugyanaz | Logikai jobb shift `(Rs & 15)` bittel. | `Z,N` frissül |
| `CMP Rd, Rs` | `EB` | 2 | ugyanaz | `Rd - Rs` csak flag-ekhez. | `Z,N,C` frissül |
| `LOAD8 Rd, [Ra]` | `EC` | 2 | `d=Rd,s=Ra` | `Rd = zero_extend(mem8[Ra])` | változatlan |
| `LOAD16 Rd, [Ra]` | `ED` | 2 | `d=Rd,s=Ra` | `Rd = mem16[Ra]` | változatlan |
| `STORE8 [Ra], Rs` | `EE` | 2 | `d=Ra,s=Rs` | `mem8[Ra] = low8(Rs)` | változatlan |
| `STORE16 [Ra], Rs` | `EF` | 2 | `d=Ra,s=Rs` | `mem16[Ra] = Rs` | változatlan |

A regiszterpár-bájt felső két bitjének nullának kell lennie; különben a VM invalid-register kódolási hibát jelez.

## Abszolút vezérlésátadó utasítások

Mindegyik három bájt: opcode + little-endian `addr16`.

| Mnemonik | Hex opcode | Definíció |
|---|---:|---|
| `JMP addr16` | `F0` | `PC = addr16` |
| `CALL addr16` | `F1` | A következő utasítás címének push-a, majd `PC = addr16`. |
| `JZ addr16` | `F2` | Ugrás, ha `Z=1`. |
| `JNZ addr16` | `F3` | Ugrás, ha `Z=0`. |
| `JC addr16` | `F4` | Ugrás, ha `C=1`. |
| `JNC addr16` | `F5` | Ugrás, ha `C=0`. |
| `JN addr16` | `F6` | Ugrás, ha `N=1`. |
| `JNN addr16` | `F7` | Ugrás, ha `N=0`. |

## Post-increment indirekt memóriautasítások

Mindegyik két bájt: opcode + `00dddsss`. Szándékosan csak a nagy értékű lineáris memóriajárás kap ilyen módot; nincs általános komplex címzésrendszer.

| Assembly szintaxis | Belső mnemonik | Hex | Pair jelentés | Definíció |
|---|---|---:|---|---|
| `LOAD8 Rd, [Ra+]` | `LOAD8P` | `F8` | `d=Rd,s=Ra` | `Rd=zero_extend(mem8[Ra]); Ra+=1` |
| `STORE8 [Ra+], Rs` | `STORE8P` | `F9` | `d=Ra,s=Rs` | `mem8[Ra]=low8(Rs); Ra+=1` |
| `LOAD16 Rd, [Ra+]` | `LOAD16P` | `FA` | `d=Rd,s=Ra` | `Rd=mem16[Ra]; Ra+=2` |
| `STORE16 [Ra+], Rs` | `STORE16P` | `FB` | `d=Ra,s=Rs` | `mem16[Ra]=Rs; Ra+=2` |

Post-increment loadnál `Rd` és `Ra` külön regiszter legyen. Store-nál a source/address regiszter lehet azonos.

`FC..FF` a pre-decrement memóriajáró család.

## Flag-definíciók

- `Z`: az eredmény nulla.
- `N`: az eredmény bit15-je egy.
- `C` összeadás után: carry-out bit15-ből.
- `C` kivonás/compare után: **1 = nincs borrow**, 0 = borrow.

Load, store, move, stack művelet, branch, call és return nem módosít flag-et, hacsak fent nincs külön jelölve.

## Endianness és aritmetika

- 16 bites memóriaértékek és instruction immediate-ek little-endian sorrendűek.
- Az aritmetika modulo 65536 wrap-el, kivéve az explicit trapet okozó műveleteket (0-val osztás/modulo).
- A shift műveletek logikaiak, nem aritmetikaiak.

## Pre-decrement memóriabejárók

Ezek a post-increment formák párjai, és a hátrafelé `memmove` ciklust költségszimmetrikussá teszik az előrefelé iránnyal. A címregiszter a hozzáférés előtt csökken.

| Assembly | Opcode | Bájt | Szemantika |
|---|---:|---:|---|
| `LOAD8 Rd,[-Ra]` | `FC` | 2 | `Ra=Ra-1; Rd=mem8[Ra]` |
| `STORE8 [-Ra],Rs` | `FD` | 2 | `Ra=Ra-1; mem8[Ra]=Rs` |
| `LOAD16 Rd,[-Ra]` | `FE` | 2 | `Ra=Ra-2; Rd=mem16[Ra]` |
| `STORE16 [-Ra],Rs` | `FF` | 2 | `Ra=Ra-2; mem16[Ra]=Rs` |

Loadnál az adat- és címregiszter különböző legyen.

## Zero-page compiler formák

| Hex | Utasítás | Bájt | Jelentés |
|---|---|---:|---|
| `03` | `ZLOAD8 addr8` | 2 | `R0 = mem8[addr8]` |
| `04` | `ZLOAD16 addr8` | 2 | `R0 = mem16[addr8]` |
| `05` | `ZSTORE8 addr8` | 2 | `mem8[addr8] = R0.low` |
| `06` | `ZSTORE16 addr8` | 2 | `mem16[addr8] = R0` |

Az implicit `R0` szándékos: már eleve ez az SVM-C expression/result regisztere, így az általános-regiszteres zero-page kódolás sokkal több opcode-helyet igényelne kevés ismétlődő előnyért.

## Megszakításvezérlés

| Hex | Utasítás | Bájt | Hatás |
|---:|---|---:|---|
| `07` | `EI` | 1 | globális interrupt-enable beállítása |
| `08` | `DI` | 1 | globális interrupt-enable törlése |
| `09` | `IRET` | 1 | mentett status/control és `PC` visszaállítása a hardveres stackről |

Interrupt belépés menti `PC`-t és a státuszt, törli az interrupt-enable állapotot, majd az MMIO-ban beállított IRQ-vektorra ugrik. A pending forrásokat `IRQ_ACK`-kal kell nyugtázni; az `IRET` nem nyugtáz automatikusan.

## Integer DSP kiterjesztés

| Utasítás | Hex kódolás | Jelentés |
|---|---|---|
| `ASR1 Rn` | `0A rr` | `Rn` aritmetikai jobbra shiftje egy bittel. |
| `MULQ15 Rd,Rs` | `0B 00dddsss` | Signed Q15 szorzás; eredmény `Rd`-be. |

A `MULQ15` signed 16 bites operandusokat, 32 bites köztes értéket és aritmetikai `>>15`-öt használ; a `0x8000 * 0x8000` egyedi túlcsordulás `0x7FFF`-re telítődik.

## Külön videó-címtér kiterjesztés

A videómemória külön 16 bites, csak adatként elérhető címtér. A regiszteres ISA hárombájtos `0C ss pp` formát használ, ahol `ss` a videó subopcode, `pp=00dddsss` pedig a normál regiszterpár-bájt. Ezek soha nem a rendszer-RAM-ot érik el, és instruction fetch nincs a videótérből.

| Mnemonik | Hex | Szemantika |
|---|---|---|
| `VLOAD8 Rd,[Ra]` | `0C 00 pp` | `Rd = video8[Ra]` |
| `VLOAD16 Rd,[Ra]` | `0C 01 pp` | `Rd = video16[Ra]` |
| `VSTORE8 [Ra],Rs` | `0C 02 pp` | `video8[Ra] = Rs` |
| `VSTORE16 [Ra],Rs` | `0C 03 pp` | `video16[Ra] = Rs` |
| `VLOAD8P Rd,[Ra+]` | `0C 04 pp` | byte load, majd `Ra += 1` |
| `VLOAD16P Rd,[Ra+]` | `0C 05 pp` | word load, majd `Ra += 2` |
| `VSTORE8P [Ra+],Rs` | `0C 06 pp` | byte store, majd `Ra += 1` |
| `VSTORE16P [Ra+],Rs` | `0C 07 pp` | word store, majd `Ra += 2` |
| `VLOAD8M Rd,[-Ra]` | `0C 08 pp` | `Ra -= 1`, majd load |
| `VLOAD16M Rd,[-Ra]` | `0C 09 pp` | `Ra -= 2`, majd load |
| `VSTORE8M [-Ra],Rs` | `0C 0A pp` | `Ra -= 1`, majd store |
| `VSTORE16M [-Ra],Rs` | `0C 0B pp` | `Ra -= 2`, majd store |

## Többwordös integer segédutasítások

| Utasítás | Kódolás | Hatás |
|---|---|---|
| `ADC Rd,Rs` | `0D 00 00dddsss` | `Rd = Rd + Rs + C`, `C`=carry-out |
| `SBC Rd,Rs` | `0D 01 00dddsss` | `Rd = Rd - Rs - (1-C)`, `C=1` = no borrow |
| `MULHU Rd,Rs` | `0D 02 00dddsss` | unsigned `Rd*Rs` felső 16 bitje |
| `RCR1 Rd` | `0D 03 rr` | jobbra forgatás carry-n keresztül |

A `SHL1` a régi bit15-öt írja `C`-be, a `SHR1` a régi bit0-t. Hardveres floating point nincs.
