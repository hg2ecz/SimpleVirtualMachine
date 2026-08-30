# SVM ISA család – aktuális referencia

Ez a dokumentum a kilenc implementált CPU-architektúra aktuális, közös referenciaindexe. A részletes opcode-táblák az `svm_asm/docs/<cpu>/INSTRUCTION_REFERENCE_HU.md`, illetve az újabb ISA-k normatív specifikációi a `docs/*_ISA_SPEC_HU.md` fájlokban találhatók.

| Cél | Operandusmodell | Általános adatállapot | Kiemelt ár–érték döntés |
|---|---|---|---|
| `stack` | 0-címes stack | data stack + `TOS/NOS` lazy cache + minimális `C` | assembly/Forth műveletek megmaradnak; `UMUL`, `ADC/SBC/RCR1`; kétcellás stack-cache |
| `accumulator` | 1-címes | `A` + `X/Y` címregiszter | implicit-A műveletek, integer carry segédek |
| `memreg` | working register + file | `W`, file, `FSR0/1` | hot `ADD/AND`; compiler scratch `0x000E..0x000F` |
| `register` | regiszter-regiszter | `R0..R7` | compact `AND` `R0..R3`; natív `SUBI` flag-helyesség miatt |
| `loadstore` | strict 3-címes load/store | `R0..R7` | külön RAM/VRAM load-store; natív hosszú `SUBI16` |
| `regmem` | 2-címes register-memory | `R0..R7` | descriptoros reg/mem/immediate source, kevés opcode-család |
| `memory2memory` | memory-to-memory | memória + `A0..A3` címregiszter | descriptoros memóriaoperandusok, közvetlen RMW |

## Közös integer minimum

Ahol az operandusmodell természetesen támogatja, az ISA-k biztosítják az `AND/OR/XOR/NOT` logikai műveleteket, 16 bites összeadást/kivonást, szorzást/osztást/modulót, shifteket, valamint a többwordös kódhoz kis hardverköltségű `ADC/SBC/MULHU/RCR1` jellegű segítséget. A Stack ugyanennek megfelelően minimális carry állapotot és `UMUL (a b -- lo hi)` formát használ.

Hardveres `f16/f32`, 32 bites ALU és CLZ nincs. A lebegőpontos aritmetika szoftveres könyvtár.

## Aktuális kódolási döntések

### Register

- `B0..BF`: compact `AND Rd,Rs` az `R0..R3` regiszterek között.
- `XOR` továbbra is normál teljes ALU-művelet.
- `D0..D7`: natív `SUBI Rd,imm16`; nem alakítható szemantikahűen `ADDI -imm` aliasra, mert a `C` flag jelentése eltérhet.

### MemReg

- `C0..CF`: hot `ADD f,W`.
- `D0..DF`: hot `ADD f,F`.
- `E0..EF`: hot `AND f,W`.
- `F0..FF`: hot `AND f,F`.
- `XOR` normál ALU-formában megmarad.

### Load/Store

A `SUBI` hosszú-immediate major `9`, function `3` külön dekódja (`SUBI16`) a helyes `C = no-borrow` szemantikát őrzi meg. A gépben nincs auto-increment load/store.

## Assembly-orientált utasítások

Az `assembly-oriented` jelölés nem deprecated státusz. A Stack `NIP/TUCK/2DUP/2DROP`, `PICK/ROLL` és `DO/?DO/I/J/LOOP/+LOOP/LEAVE/UNLOOP` családja főként kézi stack/Forth assembly olvashatósága és kódsűrűsége miatt marad a támogatott ISA-ban.

## Belt16

| target | modell | operandusállapot | fő sajátosság |
|---|---|---|---|
| `belt` | implicit-result belt | `b0..b7` legutóbbi eredmények | minden értéktermelő művelet új `b0`-t hoz létre |
| `tta` | transport-triggered | `R0..R7` + funkcionális egység portok | a célportra történő adatmozgatás triggereli a műveletet |

A Belt16 részletes normatív leírása: `docs/BELT_ISA_SPEC_HU.md`, assembler kézikönyve: `svm_asm/docs/belt/`.

A TTA16 normatív leírása: `docs/TTA_ISA_SPEC_HU.md`, assembler kézikönyve: `svm_asm/docs/tta/`. A TTA16-ban az ALU- és memória-műveleteket explicit transzportok indítják; nincs közvetlen `ADD Rd,Rs` core utasítás.


## Stack mikroarchitektúra

A Stack ISA programozói modellje változatlan, de a referencia runtime/hardvermodell kétcellás `TOS/NOS` lazy stack-cache-t használ. Ez csökkenti a data-stack RAM-forgalmat új opcode nélkül. Részletes indoklás: `docs/ARCHITECTURE_DESIGN_RATIONALE_HU.md`.
