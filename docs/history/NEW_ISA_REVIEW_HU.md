# Történeti tervezési felülvizsgálat – Új SVM ISA-k implementáció előtti felülvizsgálata

## Összegzés

A Load/Store és Memory-to-Memory v0.1 specifikációkat a meglévő akkori optimalizáló SVM-C nyelvi részhalmaz és a jelenlegi példaprogramok alapján vizsgáltuk felül. A cél az volt, hogy implementálás előtt kiderüljön, hiányzik-e olyan primitív, amely a vizsgált programokat nem vagy csak mesterséges kerülővel teszi megvalósíthatóvá.

A felülvizsgálat eredménye: **mindkét ISA implementálható, de a v0.1 Load/Store specifikációban egy funkcionális és egy költséghatékonysági hiány volt; a Memory-to-Memory ISA-nál pedig az ABI és a kis word-immediate forma pontosítása volt indokolt.** Ezeket a v0.2 specifikációk már tartalmazzák.

## Vizsgált programminták

- `svm_c/examples/hello.sc` és VT100 output;
- `svm_c/examples/video.sc`;
- `svm_c/examples/fft_q15.sc`;
- `svm_c/examples/optimization.sc`;
- register és accumulator `memmove.asm`;
- ciklusok, tömbindexelés, függvényhívás, kis konstansok, MMIO és VRAM elérés.

## Load/Store v0.1 -> v0.2

### Kötelező javítás: DIV/MOD

A jelenlegi SVM-C támogatja a `/` és `%` operátorokat, az FFT pedig futásidőben használja a `16 / len` kifejezést. A v0.1 Load/Store ISA-ban nem volt osztás vagy maradékképzés.

A v0.2 major `C` háromoperandusos arithmetic extensionként rögzíti:

- `DIVU Rd,Ra,Rb`;
- `MODU Rd,Ra,Rb`;
- `MULQ15 Rd,Ra,Rb`.

Ezzel a jelenlegi teljes unsigned SVM-C aritmetika természetesen lowerelhető.

### Ár–érték javítás: LDI6

Hardwired zero regiszter nélkül a kis konstansok előállítása a v0.1-ben rendszeresen 4 bájtos `LDI imm16` formát igényelt volna. Közben a külön `SUBI` opcode kevés új értéket adott, mert az `ADDI` signed immediate már negatív értéket is kezel.

A v0.2-ben a kis konstansokra `LDI6 Rd,0..63` került be. A korai tervezetben a `SUBI` még `ADDI -imm` assembler-alias volt, de a későbbi carry/no-borrow felülvizsgálat ezt hibásnak találta: a numerikus eredmény ugyanaz, a megfigyelhető `C` flag szemantikája viszont nem. A jelenlegi implementáció ezért a hosszú-immediate családban külön `SUBI16` dekódot használ.

### Szándékosan meg nem változtatott pontok

Nem került be auto-increment `LD/ST` vagy `VLD/VST`. A lineáris copy ezért több utasítást igényel. Ez nem hiány, hanem a strict load/store tervezési elv egyik fő mérési költsége.

## Memory-to-Memory v0.1 -> v0.2

### Kis word konstansok

Az `F2` immediate8 descriptor word műveletben most zero-extended `0..255` forrásként is használható. Így a gyakori `ADD16 x,1`, `CMP16 i,16` jellegű műveletekhez nem kell teljes 16 bites immediate extension.

Ez nem új címzési mód, csak a meglévő descriptor jobb kihasználása.

### C ABI pontosítása

A jelenlegi SVM-C a változókat és lokálisokat statikusan allokálja, ezért a Memory-to-Memory gépre nem érdemes mesterségesen klasszikus stack-frame ABI-t erőltetni. A v0.2 rögzíti:

- statikus paraméterhelyek;
- függvényenként statikus return cella;
- compiler-owned zero-page scratch;
- `A0..A3` csak címregiszter marad;
- az akkori v0.2 tervben külön hardware `SP` csak control stackre.

> **Történeti megjegyzés:** ezt a pontot a v2.3.17 implementáció felülírta. A kész Memory-to-Memory CPU-ban nincs külön rejtett `SP`; `A3` az egységes stack pointer a compiler temporary-k, `CALL/RET` és IRQ számára. Így a gép továbbra is valódi adatregiszter nélküli memory-to-memory architektúra, de két egymástól független RAM-verem nem tud egymásra írni.

Így a gép továbbra is valódi adatregiszter nélküli memory-to-memory architektúra.

## Kézi lowering eredménye

### Lineáris copy

Load/Store:

```asm
LD8  R3,[R0+0]
ST8  [R1+0],R3
ADDI R0,1
ADDI R1,1
ADDI R2,-1
BNZ  loop
```

Memory-to-Memory:

```asm
MOV8  [A1+],[A0+]
DEC16 count
BNZ   loop
```

A különbség pontosan azt a tervezési kompromisszumot mutatja, amely miatt mindkét ISA megtartása értékes.

### Framebuffer

Mindkét ISA a közös külön VRAM-modellt használja. Új MMIO vagy memory-mapped framebuffer nem szükséges. A Load/Store explicit pointerléptetést, a Memory-to-Memory pointer descriptoros auto-incrementet használ.

### FFT

A v0.2 után mindkét ISA rendelkezik a szükséges primitívekkel: változó shift, unsigned divide/modulo, Q15 multiply, tömbcímzés és 16 bites aritmetika. A Memory-to-Memory gépnél várható több scratch-memory forgalom, a Load/Store gépnél több explicit load/store és címképző utasítás. Ezek mérendő eredmények, nem specifikációs hibák.

## Implementációs döntés

A v0.2 specifikációk alapján **nem indokolt további ISA-bővítés az assembler/runtime implementáció előtt**. Különösen nem indokolt:

- auto-incrementet adni a Load/Store géphez;
- általános adatregisztert adni a Memory-to-Memory géphez;
- közös CPU/VRAM címtérre váltani;
- további DSP/SIMD utasításokat bevezetni.

Ezek már elmosnák azt a különbséget, amelyet a két architektúrával mérni szeretnénk.

## Következő fejlesztési lépés

A dokumentum ezen pontja történeti: a Load/Store és Memory-to-Memory assembler/runtime/C támogatás azóta elkészült. Az aktuális állapotot az `IMPLEMENTATION_STATUS_HU.md` írja le.
