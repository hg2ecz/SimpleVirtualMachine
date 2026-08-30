# Load/Store assembly programozási kézikönyv

A gép célja a tiszta RISC/load-store modell: számolás regisztereken, memóriaelérés külön utasítással.

```asm
MOVI R1, 0x1000
LOAD16 R0, [R1+0]
ADDI R0, 1
STORE16 [R1+0], R0
```

Többwordös aritmetikához a `C` láncolható:

```asm
ADD  R0,R0,R2
ADC  R1,R1,R3
```

Jobbra 32 bites shift:

```asm
SHR1 R1
RCR1 R0
```

A `SUBI` natív hosszú-immediate kivonás; nem célszerű `ADDI -imm` formára kézzel átírni, ha a következő kód a `C` no-borrow állapotot használja.

A VRAM külön címtér, ezért `VLOAD*`/`VSTORE*` kell. A platform MMIO címei a közös `../../../docs/PLATFORM_HU.md` dokumentumban vannak.
