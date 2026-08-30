# Memória-regiszteres CPU utasításreferencia

## Kódolás, hossz és végrehajtási idő – gyors referencia

A ciklusidők a `../../../svm_rt/docs/CYCLE_MODEL.md` modellt követik. Minden fetch-elt bájt 1 ciklus; a byte adat-hozzáférés 1, a word hozzáférés 2 ciklus, az iteratív aritmetikai műveletek pedig hozzáadják a megadott belső költséget.

| Assembly forma | Hex kódolás | Bájt | Ciklus | Megjegyzés |
|---|---|---:|---:|---|
| `NOP`, `HALT`, W-unary, W/FSR transfer, `EI`, `DI`, `ASR1W` | `00,01,05..0F,19,1A` | 1 | 1 | regiszter/belső |
| `RET`, `PUSHW`, `POPW` | `02..04` | 1 | 3 | egy 16 bites stack-hozzáférés |
| `IRET` | `1B` | 1 | 5 | két stack read |
| immediate `LDI/FSR0I/FSR1I/ADDI/...` | `10..18 lo hi` | 3 | 3 | immediate fetch |
| direct `MOV8 f,W` / `MOV8 W,f` | `20/21 ff` | 2 | 3 | byte file-hozzáférés |
| direct `MOV16 f,W` / `MOV16 W,f` | `22/23 ff` | 2 | 4 | word file-hozzáférés |
| direct ALU `... f,W`, `CMP f` | páros/direct W formák | 2 | 4 | word read |
| direct ALU `... f,F`, `INC f`, `DEC f` | write-back formák | 2 | 6 | word read + word write |
| FSR byte load/store, hold/+/- | `30,32,34,36,38,3A,3C,3E,40,42,44,46` | 1 | 2 | byte hozzáférés |
| FSR word load/store, hold/+/- | `31,33,35,37,39,3B,3D,3F,41,43,45,47` | 1 | 3 | word hozzáférés |
| `SHL f,W`, `SHR f,W` | `48/49 ff` | 2 | 5 | word read +1 belső |
| `MUL/DIV/MOD f,W` | `4A/4B/4C ff` | 2 | 20 | word read +16 belső |
| `MULQ15 f,W` | `4D ff` | 2 | 21 | word read +17 belső |
| hosszú `JMP/Jcc addr16` | `60,62..67 lo hi` | 3 | 3 | nincs branch büntetés |
| hosszú `CALL addr16` | `61 lo hi` | 3 | 5 | +stack write |
| rövid `RJMP/RJcc rel8` | `68,6A..6F dd` | 2 | 2 | assembler relaxation |
| rövid `RCALL rel8` | `69 dd` | 2 | 4 | +stack write |
| hot `MOV8` | `80..9F` | 1 | 2 | file-cím az alsó nibble-ben |
| hot `MOV16` | `A0..BF` | 1 | 3 | file-cím az alsó nibble-ben |
| hot `ADD/AND f,W` | `C0..CF`, `E0..EF` | 1 | 3 | word read |
| hot `ADD/AND f,F` | `D0..DF`, `F0..FF` | 1 | 5 | word read + write |
| videótér byte művelet | `1C ss` | 2 | 3 | +videó byte hozzáférés |
| videótér word művelet | `1C ss` | 2 | 4 | +videó word hozzáférés |

A hot-file opcode alsó nibble-je a `0x00..0x0F` file-cím. Az FSR pre-decrement/post-increment önmagában nem jelent extra ciklust.

## Programozó számára látható állapot

- `W`: 16 bites working/akkumulátor regiszter.
- `FSR0`, `FSR1`: 16 bites indirekt címregiszterek.
- `PC`, `SP`.
- flag-ek: `Z`, `N`, `C`.
- direct file tér: `0x00..0xFF` (zero page).
- hot file tér: `0x00..0x0F`; a kiválasztott műveletek a címet az opcode-ba kódolják, ezért egybájtosak.

`d=W` esetén az eredmény `W`-be kerül. `d=F` esetén visszaíródik a file operandusba.

## Fix egybájtos utasítások

| Hex | Mnemonik | Jelentés |
|---|---|---|
| `00` | `NOP` | nincs művelet |
| `01` | `HALT` | leállítás |
| `02` | `RET` | visszatérés |
| `03 / 04` | `PUSHW / POPW` | hardververem |
| `05 / 06` | `INCW / DECW` | `W +/- 1` |
| `07 / 08` | `NEGW / NOTW` | unáris W |
| `09 / 0A` | `SHL1W / SHR1W` | W shift egy bittel |
| `0B / 0C` | `W2F0 / W2F1` | `W -> FSR0/1` |
| `0D / 0E` | `F02W / F12W` | `FSR0/1 -> W` |

## Immediate utasítások (3 bájt)

| Hex | Mnemonik |
|---|---|
| `10` | `LDI imm16` |
| `11` | `FSR0I imm16` |
| `12` | `FSR1I imm16` |
| `13` | `ADDI imm16` |
| `14` | `SUBI imm16` |
| `15` | `CMPI imm16` |
| `16` | `ANDI imm16` |
| `17` | `ORI imm16` |
| `18` | `XORI imm16` |

## Közvetlen file utasítások (2 bájt)

A második bájt a zero-page file-cím.

| Hex | Forma | Szemantika |
|---|---|---|
| `20` | `MOV8 f,W` | `W = mem8[f]` |
| `21` | `MOV8 W,f` | `mem8[f] = W.low` |
| `22` | `MOV16 f,W` | `W = mem16[f]` |
| `23` | `MOV16 W,f` | `mem16[f] = W` |
| `24/25` | `ADD f,W / ADD f,F` | `W=W+F / F=F+W` |
| `26/27` | `SUB f,W / SUB f,F` | `W=W-F / F=F-W` |
| `28/29` | `AND f,W / AND f,F` | bitenkénti AND |
| `2A/2B` | `OR f,W / OR f,F` | bitenkénti OR |
| `2C/2D` | `XOR f,W / XOR f,F` | bitenkénti XOR |
| `2E` | `CMP f` | flag-ek `W-F` alapján |
| `2F` | `INC f` | 16 bites file növelése |
| `48` | `SHL f,W` | `W <<= (F & 15)` |
| `49` | `SHR f,W` | `W >>= (F & 15)` |
| `4A` | `MUL f,W` | `W *= F` |
| `4B` | `DIV f,W` | `W /= F` |
| `4C` | `MOD f,W` | `W %= F` |
| `4F` | `DEC f` | 16 bites file csökkentése |

## Indirekt memóriabejárók (1 bájt)

Az FSR0 opcode-ok `30..3B`, az FSR1 opcode-ok `3C..47`. Mindkét FSR támogat byte/word load/store műveletet három módban: változatlan cím, post-increment, pre-decrement.

Példák: `LDB0`, `LDW0+`, `STB1+`, `LDW0-`, `STW1-`.

Post-increment esetén a hozzáférés után byte-nál 1, wordnél 2 értékkel nő az FSR. Pre-decrement esetén a csökkentés a hozzáférés előtt történik.

## Vezérlésátadás

A hosszú abszolút `60..67` formák: `JMP CALL JZ JNZ JC JNC JN JNN` (3 bájt). A rövid relatív `68..6F` formák ugyanebben a sorrendben 2 bájtosak. Az assembler automatikusan a rövid formát választja, ha a signed 8 bites displacement belefér.

## Egybájtos hot file kódolások

A `0x00..0x0F` file-címeknél az opcode alsó nibble-je maga a cím:

| Tartomány | Művelet |
|---|---|
| `80..8F` | `MOV8 f,W` |
| `90..9F` | `MOV8 W,f` |
| `A0..AF` | `MOV16 f,W` |
| `B0..BF` | `MOV16 W,f` |
| `C0..CF` | `ADD f,W` |
| `D0..DF` | `ADD f,F` |
| `E0..EF` | `AND f,W` |
| `F0..FF` | `AND f,F` |

Ez szándékosan csak a nagy gyakoriságú műveletekre terjed ki. A szabad opcode-helyet nem töltjük fel pusztán azért, mert létezik. Az `XOR` normál file/working-register formában megmarad; az `AND` kapott hot kódolást, mert a jelenlegi wide-int és soft-float workloadokban a maszkolás dominál.

## Megszakításvezérlés

| Hex | Utasítás | Bájt | Hatás |
|---:|---|---:|---|
| `19` | `EI` | 1 | globális maskable interrupt engedélyezése |
| `1A` | `DI` | 1 | globális maskable interrupt tiltása |
| `1B` | `IRET` | 1 | mentett státusz/control és `PC` visszaállítása |

Ezek korábban szabad opcode-helyet használnak és csak egyetlen globális enable bitet adnak a CPU állapotához.

## Integer DSP kiterjesztés

| Utasítás | Hex kódolás | Jelentés |
|---|---|---|
| `ASR1W` | `0F` | `W` aritmetikai jobbra shiftje |
| `MULQ15 f,W` | `4D ff` | `W = q15(W * mem[ff])` |

A `MULQ15` signed 16 bites operandusokat, 32 bites köztes értéket és aritmetikai `>>15` visszaskálázást használ; a `0x8000 * 0x8000` egyedi túlcsordulás `0x7FFF`-re telítődik.

## Külön videó-címtér kiterjesztés

A `0x1C` prefix egy indirekt FSR-műveletet a külön videótérre irányít. A `00..0B` subopcode-ok az FSR0 hold/post-increment/pre-decrement byte/word load/store formái; `0C..17` az FSR1 megfelelő formái.

Példák:

| Mnemonik | Hex | Jelentés |
|---|---|---|
| `VLDB0` | `1C 00` | `video8[FSR0] -> W` |
| `VLDW0` | `1C 01` | `video16[FSR0] -> W` |
| `VSTB0+` | `1C 06` | `W -> video8[FSR0++]` |
| `VSTW0+` | `1C 07` | `W -> video16[FSR0]; FSR0+=2` |
| `VLDB0-` | `1C 08` | `--FSR0; video8[FSR0] -> W` |
| `VSTB0-` | `1C 0A` | `--FSR0; W -> video8[FSR0]` |
| `VLDB1` | `1C 0C` | `video8[FSR1] -> W` |
| `VSTB1+` | `1C 12` | `W -> video8[FSR1++]` |
| `VLDB1-` | `1C 14` | `--FSR1; video8[FSR1] -> W` |
| `VSTB1-` | `1C 16` | `--FSR1; W -> video8[FSR1]` |

## Többwordös integer segédutasítások

Az `50..55` opcode-ok: `ADC f,W`, `ADC f,F`, `SBC f,W`, `SBC f,F`, `MULHU f,W`, `RCR1W`. Az `ADC/SBC` a `C` carry-láncot viszi tovább; a `MULHU` az unsigned 16×16 szorzat felső wordjét adja. A `SHL1W/SHR1W` a kilépő bitet `C`-be írja.
