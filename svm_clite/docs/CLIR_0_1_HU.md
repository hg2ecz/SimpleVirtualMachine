# CLIR 0.1 – a C-Lite architektúrafüggetlen assemblyje

A CLIR szándékosan kicsi. Nem optimalizáló IR, nincs SSA, nincs fizikai regiszter, nincs stack frame, nincs target-specifikus utasítás. A célja az, hogy egy C-Lite program strukturált elemei egyszerű, assembly-szerű műveletekre bomoljanak.

## Értékek

A `%0`, `%1`, ... nevek virtuális ideiglenes értékek. A CLIR nem írja elő, hol élnek fizikailag: a Stack backend például közvetlenül az adatvermen tartja őket, más ISA használhat memóriát vagy saját természetes erőforrást. Nincs általános regiszterallokáció.

```text
const.u16 %0, 10
load.u16 %1, counter
add.u16 %2, %0, %1
store.u16 counter, %2
```

## Típusutótagok

A műveletek típusa explicit:

```text
.bool .u8   .i8   .u16   .i16
```

Az összehasonlítás operandustípusa is látszik, például `lt.i16`. Az összehasonlítás eredménye `bool`, fizikailag 0 vagy 1.

## Memória

```text
load.T       %dst, variable
store.T      variable, %src
addr         %dst, variable
index        %dst, %base, %index, element_size
loadmem.T    %dst, %address
storemem.T   %address, %src
loadmemv.T   %dst, %address
storememv.T  %address, %src
```

A `v` változat MMIO/volatile memóriaeléréshez való.

## Aritmetika és bitműveletek

```text
add.T sub.T mul.T div.T mod.T
and.T or.T xor.T shl.T shr.T
neg.T not.T
```

Nincs konstanshajtás vagy más optimalizáció: a forrásban szereplő műveletek láthatók maradnak az IR-ben.

## Összehasonlítás

```text
eq.T ne.T lt.T le.T gt.T ge.T
```

## Vezérlés

```text
label:
jz %condition, label
jmp label
call.T %result = function(%a, %b)
call function(%a, %b)
ret %value
ret
drop %value
```

A `drop` explicit módon jelzi, hogy egy kifejezés eredményére nincs szükség. Stack targeten natív `DROP`, olyan backenden pedig, ahol a temp eleve statikus helyen él, nem igényel utasítást.

Az `if` és `while` kizárólag ezekre a műveletekre bomlik. A C-Lite-ban nincs `goto`, de a CLIR-ben természetesen vannak címkék és ugrások.

## Példa: while

C-Lite:

```c
u16 i = 0;
while (i < n) {
    i = i + 1;
}
```

CLIR alakja:

```text
const.u16 %0, 0
store.u16 i, %0
while_test_0:
load.u16 %1, i
load.u16 %2, n
lt.u16 %3, %1, %2
jz %3, while_end_1
load.u16 %4, i
const.u16 %5, 1
add.u16 %6, %4, %5
store.u16 i, %6
jmp while_test_0
while_end_1:
```

## Tanulási sorrend

1. `const`, `load`, `store`
2. aritmetika
3. `addr`, `index`, `loadmem`, `storemem`
4. `jz`, `jmp`
5. `call`, `ret`
6. ugyanazon CLIR összevetése két külön target assemblyjével

A CLIR 0.1-et 1.0 előtt stabil műveletkészletnek tekintjük. Új nyelvi kényelmi elem csak akkor indokolt, ha ehhez nem kell új, bonyolult IR-mechanika.

## Bool

```text
const.bool %0, 1
load.bool %1, ready
store.bool ready, %0
```

A `bool` CLIR-ben logikai 1 bites jelentésű, de a közvetlen backend memóriában egy byte-ot használ. A CLIR nem ír elő flag/carry megvalósítást.
