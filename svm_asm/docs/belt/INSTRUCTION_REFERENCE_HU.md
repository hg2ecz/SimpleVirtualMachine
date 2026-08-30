# Belt16 utasításreferencia

## Vezérlés

`NOP HALT RET EI DI IRET JMP JZ JNZ JC JNC JN JNN CALL`

## Érték-előállítás

`LDI imm16`, `LD8A addr`, `LD16A addr`, `LD8 [bN]`, `LD16 [bN]`, `VLD8 [bN]`, `VLD16 [bN]` és `POP` új `b0` eredményt hoz létre.

## Tárolás

`ST8A addr,bN`, `ST16A addr,bN`, `ST8 [bA],bV`, `ST16 [bA],bV`, `VST8 [bA],bV`, `VST16 [bA],bV` és `PUSH bN` nem termel belt-eredményt.

## Bináris ALU

`ADD SUB AND OR XOR MUL DIV MOD SHL SHR CMP ADC SBC MULHU MULQ15 bA,bB` új `b0` eredményt hoz létre.

`SUB/CMP` esetén `C=1` jelentése no-borrow. `ADC/SBC` a carry-láncot használja.

## Unáris ALU

`PASS NOT NEG ASR1 SHL1 SHR1 RCR1 bA` új `b0` eredményt hoz létre.

`SHL1`: régi bit15 -> C. `SHR1`: régi bit0 -> C. `RCR1`: régi C -> bit15, régi bit0 -> C.
