# Belt16 assembly programozási kézikönyv

A Belt16 a nyolc legutóbbi 16 bites eredményt `b0..b7` néven tartja; `b0` mindig a legfrissebb. Minden eredményt előállító utasítás egy hellyel öregíti a korábbi értékeket. Nincs általános célú regiszterfájl.

```asm
.load 0x0100
.entry start
.proc start
    LDI 10
    LDI 20
    ADD b1,b0
    ST16A 0x6000,b0
    HALT
.endproc
```

Abszolút memória: `LD8A/LD16A`, `ST8A/ST16A`. Pointeres memória: `LD8/LD16 [bN]`, `ST8/ST16 [bA],bV`. Videómemória: `VLD8/VLD16`, `VST8/VST16`.

`PUSH bN` és `POP` elsősorban fordítói/assembly kényelmi primitív. `POP` eredményt termel, ezért beltre kerül.

A `CMP bA,bB` szintén eredményt termel (`a-b`) és frissíti a `Z/N/C` flag-eket; utána `JZ/JNZ/JC/JNC/JN/JNN` használható.

## Eljárásblokkok és nem használt kód eltávolítása

A nyilvános/hívható rutinokat `.proc NAME` ... `.endproc` blokkokban célszerű írni. Az `.entry NAME` a program belépési eljárását elérhetőségi gyökérré teszi; a `.keep NAME` hardveres callback vagy önálló könyvtári töredék explicit megtartására szolgál. Az assembler az `.include` és `.equ` kifejtése után eltávolítja azokat a `.proc` blokkokat, amelyek ezekből a gyökerekből vagy az élő kódban szereplő szimbolikus hivatkozásokból nem érhetők el. Az eljáráson belüli közönséges címkék továbbra is helyi vezérlési címkék, nem külön elhagyható eljárások.

## Grafikai könyvtár

A `graphics.asm` exportálja a gyors `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, `vline` primitíveket, továbbá a magasabb szintű `line`, `rect`, `fillrect`, `circle`, `fillcircle` eljárásokat. ABI: b0=szín; paletta: b0..b3; putpixel: b0=x, b1=y; clear: b0=szín; hline: b0=x0,b1=x1,b2=y; vline: b0=x,b1=y0,b2=y1. A nem használt eljárásokat a procedure-GC eltávolítja.

Az öt vagy több logikai paramétert igénylő alakzatok minden ISA-n ugyanazt a 16 bites grafikai paraméterblokkot használják: `GFX_X0=0x00C0`, `GFX_Y0=0x00C2`, `GFX_X1=0x00C4`, `GFX_Y1=0x00C6`, `GFX_W=0x00C8`, `GFX_H=0x00CA`, `GFX_R=0x00CC`, `GFX_COLOR=0x00CE`. A `0x00B0..0x00BE` tartomány egyes targeteken belső virtuálisregiszter-scratch; a `0x00D0..0x00FA` tartomány további grafikai scratch/current-color terület. A `graphics.asm` használatakor ezért a teljes `0x00B0..0x00FA` tartomány fenntartott. `line` az `(x0,y0,x1,y1,color)`, `rect/fillrect` az `(x,y,w,h,color)`, `circle/fillcircle` a `(cx,cy,r,color)` mezőket olvassa. A procedure-GC a nem használt alakzatrutinokat és függőségeiket eltávolítja.

