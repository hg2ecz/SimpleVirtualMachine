# Belt16 assembly programozási kézikönyv

A Belt16 a nyolc legutóbbi 16 bites eredményt `b0..b7` néven tartja; `b0` mindig a legfrissebb. Minden eredményt előállító utasítás egy hellyel öregíti a korábbi értékeket. Nincs általános célú regiszterfájl.

```asm
.load 0x0100
.entry start
start:
    LDI 10
    LDI 20
    ADD b1,b0
    ST16A 0x6000,b0
    HALT
```

Abszolút memória: `LD8A/LD16A`, `ST8A/ST16A`. Pointeres memória: `LD8/LD16 [bN]`, `ST8/ST16 [bA],bV`. Videómemória: `VLD8/VLD16`, `VST8/VST16`.

`PUSH bN` és `POP` elsősorban fordítói/assembly kényelmi primitív. `POP` eredményt termel, ezért beltre kerül.

A `CMP bA,bB` szintén eredményt termel (`a-b`) és frissíti a `Z/N/C` flag-eket; utána `JZ/JNZ/JC/JNC/JN/JNN` használható.
