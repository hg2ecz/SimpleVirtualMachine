# SVM Register-Memory CPU – ISA specifikáció (implementált v1)

## 1. Cél

A Register-Memory CPU a kétoperandusos, általános regiszter–memória ISA-k összehasonlítási pontja. Az ALU célja mindig általános regiszter, a source pedig lehet regiszter, CPU-memória vagy immediate. Így például az `ADD R0,[R1+4]` egyetlen utasítás, miközben általános memória–memória ALU nincs.

A modell a strict Load/Store és a teljes Memory-to-Memory gép közé esik, és nem azonos a MemReg implicit `W` working-register modelljével.

## 2. Közös platform

- 64 KiB CPU-címtér;
- külön 16 KiB VRAM;
- közös MMIO, timer, IRQ, konzol és karaktergenerátor;
- nincs guest-visible System ROM;
- 16 bites little-endian adatmodell;
- instruction fetch csak CPU-memóriából.

## 3. Programmer-visible állapot

- `R0..R7`: nyolc 16 bites általános regiszter;
- `PC`: 16 bit;
- `Z`, `N`, `C` flag;
- interrupt-enable állapot.

A korábbi v0.1 tervezet 4 GPR-t javasolt. Az implementáció 8 GPR-t használ. Ennek oka, hogy a Register-Memory gép így közvetlenebbül összevethető a Register és Load/Store géppel, a C backendnek pedig nem kell mesterséges spillt bevezetnie csak a kisebb regiszterfájl miatt. A plusz regiszterek nem változtatják meg az operandusmodell lényegét.

Az implementált ABI `R6`-ot egységes stack pointerként használja; resetkor `R6=0xFF00`, és lefelé nő a `0xFB00..0xFEFF` RAM-tartományban. A compiler `PUSH/POP` ideiglenesei, a `CALL/RET` visszatérési címek és az interrupt mentések ugyanazon `R6` veremen osztoznak. Nincs külön rejtett control-stack pointer.

## 4. Alapelv

1. Kétoperandusos ALU: `Rd = Rd op src`.
2. ALU destination kizárólag GPR.
3. `src` lehet GPR, memória vagy immediate.
4. Egy általános ALU-utasítás legfeljebb egy adatmemória-hozzáférést végez.
5. Store külön művelet.
6. Auto-update címzés csak load/store/video move műveletnél engedett, ALU source-ként nem.
7. VRAM külön címtér és külön opcode-család.

## 5. Implementált source descriptor

A descriptor változó hosszúságú.

| Descriptor | Jelentés | Extension |
|---|---|---|
| `00..07` | `R0..R7` | nincs |
| `10..17` | `[R0]..[R7]` | nincs |
| `18..1F` | `[R0+]..[R7+]` | nincs; load/store/video move |
| `28..2F` | `[-R0]..[-R7]` | nincs; load/store/video move |
| `20..27` | `[R0+off8]..[R7+off8]` | signed byte |
| `E0` | zero-page direct | `addr8` |
| `E1` | absolute16 | `addr16` |
| `F0` | zero-extended immediate8 | `imm8` |
| `F1` | immediate16 | `imm16` |

A memória-source assembly szintaxisa szögletes zárójelet használ. Például `ADD R0,[0x1234]` memóriaértéket olvas, míg `ADD R0,0x1234` immediate értéket használ.

## 6. ALU

A bináris műveletek implementált készlete:

`MOV`, `ADD`, `SUB`, `AND`, `OR`, `XOR`, `CMP`, `MUL`, `DIV`, `MOD`, `SHL`, `SHR`, `MULQ15`.

Példák:

```asm
ADD R0,R1
ADD R0,[R2+4]
SUB R3,7
CMP R0,[0x0040]
```

A `DIV/MOD` unsigned 16 bites. Nulla osztó trap. `MULQ15` a többi SVM géppel azonos signed Q15 művelet.

## 7. Unary műveletek

Implementált: `NOT`, `NEG`, `INC`, `DEC`, `ASR1`, `SHL1`, `SHR1`.

Az `INC/DEC` külön rövid műveletként marad, mert pointer- és cikluskódban gyakori, és nem igényel source descriptort.

## 8. Load/store

```asm
LOAD8  R0,[R1]
LOAD16 R2,[R3+6]
STORE8 [R1+],R0
STORE16 [-R4],R2
```

A byte load zero-extended. A word hozzáférés little-endian. Az auto-update byte műveletnél 1, word műveletnél 2.

Az assembler kompatibilitási pseudo-opként elfogadja a `ZLOAD8/ZLOAD16/ZSTORE8/ZSTORE16` alakokat is.

## 9. Control flow

A v1 implementáció stabil, 16 bites abszolút célcímes formákat használ:

`JMP`, `JZ`, `JNZ`, `JC`, `JNC`, `JN`, `JNN`, `CALL`, továbbá `RET`, `EI`, `DI`, `IRET`, `NOP`, `HALT`.

Branch relaxation jelenleg nincs a Register-Memory assemblerben. Ez későbbi, tisztán assembler-oldali kódsűrűségi optimalizáció lehet; az ISA operandusmodelljét nem érinti.

## 10. VRAM

Implementált:

`VLOAD8`, `VLOAD16`, `VSTORE8`, `VSTORE16`.

A címző descriptor ugyanazt a pointer/offset/absolute modellt használja, de a hozzáférés a külön 16 KiB video-adattérre történik.

## 11. C backend

A közös `svm_c` frontend mindkét fordítási irány (`svm-c` és `svm-c-unopt-only`) számára azonos. A kezdeti Register-Memory lowering a kiforrott register-backend kifejezéskezelését használja, az assembler pedig a kétoperandusos műveleteket natív Register-Memory kódra fordítja. A kézi assembly már közvetlen memória-source ALU-t is használhat.

Következő optimalizációs lehetőség: a C backend egyszerű `x op variable` esetekben közvetlenül `OP Rn,[addr]` formát generálhat, kihagyva az explicit loadot. Ehhez nincs szükség ISA-változtatásra.

## 12. Ár–érték korlátok

Szándékosan nincs:

- memória–memória ALU;
- általános byte ALU-család;
- scaled-index addressing;
- auto-update ALU source;
- SIMD/FPU;
- signed DIV/MOD külön opcode;
- CPU-címezhető ROM.

## 13. Executable azonosító

Normatív v1 magic: `SVR\x01`.


## Integer segédletek

Az implementált register-memory ISA további műveletei: `ADC`=`2D`, `SBC`=`2E`, `MULHU`=`2F`, `RCR1`=`37`. A `SHL1/SHR1` a kieső bitet a C flagbe írja. Ezek többwordös integer és soft-float könyvtári kódot segítenek, hardveres floating point nélkül.
