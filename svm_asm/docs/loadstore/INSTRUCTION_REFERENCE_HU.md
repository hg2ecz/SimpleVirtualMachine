# Load/Store ISA – utasításreferencia

A Load/Store CPU 16 bites, 8 általános regiszteres (`R0..R7`) strict load/store gép. Az ALU regisztereken dolgozik; a rendszer-RAM és VRAM csak külön load/store utasításokon érhető el.

## Állapot és flag-ek

`Z` = nulla, `N` = bit15, `C` = carry / kivonásnál no-borrow. `SHL1` a régi bit15-öt, `SHR1` a régi bit0-t írja `C`-be; `RCR1` carry-n keresztül forgat jobbra.

## Magutasítások

- vezérlés: `NOP HALT RET EI DI IRET`
- adat/ALU: `MOV CMP NOT NEG ASR1 INC DEC`
- két- vagy háromoperandusos ALU: `ADD SUB AND OR XOR MUL SHL SHR`
- többwordös/fixpontos segéd: `DIV/DIVU MOD/MODU MULQ15 ADC SBC MULHU RCR1`
- literal: `MOVI/LDI`, `ADDI`, `SUBI`, `CMPI`, `ANDI`, `ORI`, `XORI`
- memória: `LOAD8 LOAD16 STORE8 STORE16`
- VRAM: `VLOAD8 VLOAD16 VSTORE8 VSTORE16` (`VLD*`/`VST*` aliasok)
- branch/call: `BRA BZ BNZ BC BNC BN BNN`, valamint `JMP JZ JNZ JC JNC JN JNN CALL`
- assembler kényelmi formák: `PUSH POP ZLOAD8/16 ZSTORE8/16`

## Immediate kódolás

Kis signed `ADDI/CMPI` érték `-32..31` rövid 6 bites formát használhat. `ANDI/ORI/XORI` kis `0..63` immediate-et használ. A teljes 16 bites `MOVI`, `ADDI`, `CMPI` kétwordös hosszú-immediate formát kap.

A `SUBI` **nem** `ADDI -imm` alias: a hosszú-immediate major `9`, function `3` külön `SUBI16` dekódot használ, mert a numerikus eredmény ugyan ekvivalens lenne, de a `C` carry/no-borrow szemantika nem.

## Tervezési minimum

Nincs auto-increment load/store. Ez szándékos: a gép a tiszta load/store kontrollpontot képviseli. Hardveres floating point, 32 bites ALU és CLZ nincs; `f16/f32` szoftveres.
