# Belt16 ISA specifikáció

## Cél

A Belt16 a nyolcadik SVM összehasonlító CPU. A tervezési alapelv az implicit eredményhely: nincs általános regiszterfájl és nincs operandusverem. Minden értéktermelő utasítás eredménye automatikusan `b0` lesz, a korábbi eredmények `b1..b7` felé öregednek.

A gép szándékosan nem Mill/VLIW megvalósítás. Egyetlen 16 bites végrehajtási út, 8x16 bites belt, közös 64 KiB CPU-címtér, külön VRAM és azonos MMIO tartozik hozzá.

## Belt szemantika

`LDI 10` után `b0=10`. Új eredmény keletkezésekor a régi `b0` lesz `b1`, a régi `b1` lesz `b2`, stb.; a régi `b7` elvész.

```asm
LDI 10       ; b0=10
LDI 20       ; b0=20, b1=10
ADD b1,b0    ; b0=30
```

A store, branch és `PUSH` nem termel belt-eredményt. A `POP` igen.

## Állapot

- 8 x 16 bites belt elem
- 16 bites PC
- minimális `Z/N/C` aritmetikai flag készlet
- külön control stack CALL/RET számára
- kis adatstack a C backend temporális mentéseihez

A flag-ek nem teszik register géppé az ISA-t; csak branch/carry szemantikát szolgálnak. A fő operandusmodell továbbra is a relatív belt-hivatkozás.

## Integer mag

`ADD SUB AND OR XOR NOT NEG MUL DIV MOD SHL SHR SHL1 SHR1 ASR1 ADC SBC MULHU MULQ15 RCR1`.

Hardveres floating point, 32 bites ALU és CLZ nincs. Az `f16/f32` továbbra is szoftveres könyvtár.

## C backend

Az első backend funkcionális, konzervatív lowering. A közös C backend virtuális temporaries-ait a `0x0000..0x000F` compiler-owned memóriába süllyeszti, majd `LD -> belt ALU -> ST` mintával dolgozik. A felhasználói statikus terület ezért Belt targeten `0x0020`-tól kezdődik.

Ez nem az ISA korlátja, hanem az első codegen egyszerűségi döntése. Később külön belt-lifetime optimalizálás hagyhatja a rövid életű eredményeket a belten.

## Zero-page compiler forms

A Belt16 a C fordító kódsűrűsége miatt rövid, 8 bites abszolút című zero-page formákat is tartalmaz:

- `ZLD8 addr8`, `ZLD16 addr8`: érték betöltése a belt elejére (`b0`),
- `ZST8 addr8,bN`, `ZST16 addr8,bN`: belt-érték tárolása a `0x0000..0x00FF` tartományba.

Ezek nem új adatút-elvet jelentenek; a normál abszolút memória-hozzáférés rövid kódolásai. A C backend elsősorban a compiler-owned temporaries és a zero-page statikus objektumok kezelésére használja őket. Egy utasítás 2 bájtos, szemben a teljes 16 bites abszolút címes formák 4 bájtjával.

A folytonos programkép nem lóghat át a `0xFF00` kezdetű MMIO-sávon. A Belt assembler ezt fordításkor hibaként jelzi, így túl nagy program nem tud futás közben perifériaregiszterekre töltődve megsérülni.
