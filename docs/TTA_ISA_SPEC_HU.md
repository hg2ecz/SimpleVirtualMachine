# SVM TTA16 ISA - specifikacio

## Cel

A TTA16 a Transport Triggered Architecture egyszerusitett, 16 bites SVM
valtozata. Nem VLIW es nem tobbutasitasos issue gep. A program alapmuvelete az
adat transzportja egy forrasbol egy celportba. Bizonyos celportokra torteno iras
inditja el a funkcionális egyseg muveletet.

A platform tobbi CPU-javal azonos 64 KiB CPU-cimteret, kulon VRAM-ot es MMIO-t
hasznal. Nincs hardveres lebegopontos egyseg.

## Allapot

- 8 darab 16 bites transport-regiszter: `R0..R7`
- `PC`, valamint `Z/N/C/I` minimalis flag/allapot
- `ALU.X` operandus-latch es `ALU.OUT` eredmenylatch
- `MEM.ADDR` es `VMEM.ADDR` cim-latch
- control stack: `0xFD00..0xFEFF`
- compiler PUSH/POP data stack: `0xFB00..0xFCFF`

## Alaputasitas

    MOV source,destination

Pelda:

    MOV R0,ALU.X
    MOV R1,ALU.ADD
    MOV ALU.OUT,R2

A masodik transport triggereli az osszeadast. Az eredmeny csak az `ALU.OUT`
forrasbol olvashato ki.

## Forrasportok

- `R0..R7`
- `ALU.OUT`
- `MEM.R8`, `MEM.R16`
- `VMEM.R8`, `VMEM.R16`
- `STACK.POP`
- `CTRL.RETADDR`, `CTRL.IRETADDR`
- `FLAGS`, `ZERO`
- 16 bites literal vagy cimke mint immediate source

## Celportok

- `R0..R7`
- `ALU.X`
- `ALU.ADD`, `ALU.ADC`, `ALU.SUB`, `ALU.SBC`
- `ALU.AND`, `ALU.OR`, `ALU.XOR`
- `ALU.MUL`, `ALU.MULHU`, `ALU.MULQ15`, `ALU.DIV`, `ALU.MOD`
- `ALU.SHL`, `ALU.SHR`, `ALU.CMP`
- `ALU.NOT`, `ALU.NEG`, `ALU.ASR1`, `ALU.SHL1`, `ALU.SHR1`, `ALU.RCR1`
- `MEM.ADDR`, `MEM.W8`, `MEM.W16`
- `VMEM.ADDR`, `VMEM.W8`, `VMEM.W16`
- `CTRL.JMP/JZ/JNZ/JC/JNC/JN/JNN/CALL/HALT/EI/DI`
- `STACK.PUSH`

## Assembler convenience formak

`NOP`, `HALT`, `EI`, `DI`, `RET`, `IRET`, `PUSH`, `POP`, `JMP`, `JZ`, `JNZ`,
`JC`, `JNC`, `JN`, `JNN` es `CALL` assembler-szintu roviditesek. Ezek nem uj
ALU/datapath funkciok; a megfelelo transzportokra fordulnak.

## Kodolás

A core transport egy 16 bites word. A source es destination port 6-6 bites
azonositot kap. Immediate source eseten a transport wordot egy 16 bites literal
koveti. Executable magic: `SVT\x01`.

## C backend

A jelenlegi backend a kozos C frontendet es Register virtualis expression
loweringet hasznalja, de a vegso TTA assembly minden ALU-, memoria- es control
muveletet explicit transportokra bont. Ez backend implementacios dontes, nem az
ISA korlatozasa.
