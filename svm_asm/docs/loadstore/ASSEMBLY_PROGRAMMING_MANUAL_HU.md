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

## Eljárásblokkok és nem használt kód eltávolítása

A nyilvános/hívható rutinokat `.proc NAME` ... `.endproc` blokkokban célszerű írni. Az `.entry NAME` a program belépési eljárását elérhetőségi gyökérré teszi; a `.keep NAME` hardveres callback vagy önálló könyvtári töredék explicit megtartására szolgál. Az assembler az `.include` és `.equ` kifejtése után eltávolítja azokat a `.proc` blokkokat, amelyek ezekből a gyökerekből vagy az élő kódban szereplő szimbolikus hivatkozásokból nem érhetők el. Az eljáráson belüli közönséges címkék továbbra is helyi vezérlési címkék, nem külön elhagyható eljárások.

## Grafikai könyvtár

A `graphics.asm` exportálja a gyors `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, `vline` primitíveket, továbbá a magasabb szintű `line`, `rect`, `fillrect`, `circle`, `fillcircle` eljárásokat. ABI: R0=szín; paletta: R0..R3; putpixel: R0=x,R1=y; clear: R0=szín; hline: R0=x0,R1=x1,R2=y; vline: R0=x,R1=y0,R2=y1. A nem használt eljárásokat a procedure-GC eltávolítja.

Az öt vagy több logikai paramétert igénylő alakzatok minden ISA-n ugyanazt a 16 bites grafikai paraméterblokkot használják: `GFX_X0=0x00C0`, `GFX_Y0=0x00C2`, `GFX_X1=0x00C4`, `GFX_Y1=0x00C6`, `GFX_W=0x00C8`, `GFX_H=0x00CA`, `GFX_R=0x00CC`, `GFX_COLOR=0x00CE`. A `0x00B0..0x00BE` tartomány egyes targeteken belső virtuálisregiszter-scratch; a `0x00D0..0x00FA` tartomány további grafikai scratch/current-color terület. A `graphics.asm` használatakor ezért a teljes `0x00B0..0x00FA` tartomány fenntartott. `line` az `(x0,y0,x1,y1,color)`, `rect/fillrect` az `(x,y,w,h,color)`, `circle/fillcircle` a `(cx,cy,r,color)` mezőket olvassa. A procedure-GC a nem használt alakzatrutinokat és függőségeiket eltávolítja.

