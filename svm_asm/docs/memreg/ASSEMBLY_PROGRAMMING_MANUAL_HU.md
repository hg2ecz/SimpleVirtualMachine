# Memory-register assembly programozási kézikönyv


> A jelenlegi videómodellben a normál rendszerterület és a videómemória két külön 16 bites címtér. A normál memóriautasítás nem lép át a videótérbe. A hiteles kiosztás: `../../../docs/PLATFORM_HU.md`.

A Memory-register CPU PIC ihletésű, költségoptimalizált 16 bites architektúra, a történelmi bankolt memória korlátai nélkül. Az aritmetika központja a W regiszter és a 0. lapos file operandus; a teljes 64 KiB címtérhez két FSR tartozik.

A javasolt felosztás: `0x0000..0x00EF` gyors változók, `0x00F0..0x00FF` fordítói/scratch terület, a programkód pedig `0x0100`-tól.

## Eljárásblokkok és nem használt kód eltávolítása

A nyilvános/hívható rutinokat `.proc NAME` ... `.endproc` blokkokban célszerű írni. Az `.entry NAME` a program belépési eljárását elérhetőségi gyökérré teszi; a `.keep NAME` hardveres callback vagy önálló könyvtári töredék explicit megtartására szolgál. Az assembler az `.include` és `.equ` kifejtése után eltávolítja azokat a `.proc` blokkokat, amelyek ezekből a gyökerekből vagy az élő kódban szereplő szimbolikus hivatkozásokból nem érhetők el. Az eljáráson belüli közönséges címkék továbbra is helyi vezérlési címkék, nem külön elhagyható eljárások.


## Destination flag modell

```asm
MOV16 0x10,W
ADD   0x12,W     ; W = W + file[0x12]
ADD   0x14,F     ; file[0x14] = file[0x14] + W
```

A legelső 16 file-címnél a gyakori műveletek egyetlen opcode-bájtra rövidülnek.

## Indirekt címzés

```asm
FSR0I source
FSR1I destination
LDB0+             ; W = *FSR0++
STB1+             ; *FSR1++ = W
```

Átfedő hátramásolásnál a pointerek a másolandó tartomány vége után indulnak:

```asm
LDB0-
STB1-
```

## Zero page és teljes memória

A direct file címzés csak `0x0000..0x00FF` területet ér el; más címhez FSR-indirekt elérés használatos. Nincs bankregiszter vagy lapváltási állapot.

## Timer / interrupt gyors referencia

A közös platform 32 bites virtuális órát, egy 16 bites timert és timer/VSYNC/billentyűzet IRQ-forrásokat biztosít a `0xFF12..0xFF1F` tartományban. A vektort és forrásmaszkot tiltott interrupt mellett célszerű beállítani; a kezelt forrást az `IRQ_ACK` (`0xFF14`) regiszteren kell nyugtázni, majd `IRET`-tel visszatérni. A normatív MMIO-szemantika: `../../../docs/PLATFORM_HU.md`.


## Utasításkódolás és végrehajtási idő

A hex opcode, utasításhossz és ciklusidő táblázatai: [INSTRUCTION_REFERENCE_HU.md](INSTRUCTION_REFERENCE_HU.md).

## Grafikai könyvtár

A `graphics.asm` exportálja a gyors `gfx_set_color`, `gfx_set_palette`, `putpixel`, `clear`, `hline`, `vline` primitíveket, továbbá a magasabb szintű `line`, `rect`, `fillrect`, `circle`, `fillcircle` eljárásokat. ABI: W=szín; paletta: FSR0 -> 4 bájtos tábla; putpixel: FSR0=x,FSR1=y; clear: W=szín; hline: FSR0=x0,FSR1=x1,W=y; vline: FSR0=x,FSR1=y0,W=y1. A nem használt eljárásokat a procedure-GC eltávolítja.

Az öt vagy több logikai paramétert igénylő alakzatok minden ISA-n ugyanazt a 16 bites grafikai paraméterblokkot használják: `GFX_X0=0x00C0`, `GFX_Y0=0x00C2`, `GFX_X1=0x00C4`, `GFX_Y1=0x00C6`, `GFX_W=0x00C8`, `GFX_H=0x00CA`, `GFX_R=0x00CC`, `GFX_COLOR=0x00CE`. A `0x00B0..0x00BE` tartomány egyes targeteken belső virtuálisregiszter-scratch; a `0x00D0..0x00FA` tartomány további grafikai scratch/current-color terület. A `graphics.asm` használatakor ezért a teljes `0x00B0..0x00FA` tartomány fenntartott. `line` az `(x0,y0,x1,y1,color)`, `rect/fillrect` az `(x,y,w,h,color)`, `circle/fillcircle` a `(cx,cy,r,color)` mezőket olvassa. A procedure-GC a nem használt alakzatrutinokat és függőségeiket eltávolítja.

