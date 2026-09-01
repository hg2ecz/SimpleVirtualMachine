# SVM C-Lite tanulási útvonal

A C-Lite célja nem az, hogy minden C nyelvi elemet megtanítson. A cél az, hogy ugyanazt a programot három egyre alacsonyabb szinten lehessen megérteni:

1. **C-Lite** – strukturált, C-szerű forrás;
2. **CLIR** – architektúrafüggetlen assembly-szerű köztes forma;
3. **target ASM** – a kiválasztott virtuális architektúra konkrét assemblyje.

Normál programozáshoz csak az első szint szükséges. A második és harmadik szint tanulásra és hibakeresésre szolgál.

## 1. Ellenőrzés kódgenerálás nélkül

```sh
svm-clite --check examples/learning_01_arithmetic.cl
```

Ez csak a nyelvi és szemantikai ellenőrzést futtatja. Jó első lépés, mert még nem kell assemblyre vagy IR-re figyelni.

## 2. Egyszerű aritmetika

C-Lite:

```c
fn add(u16 a, u16 b) -> u16 {
    return a + b;
}
```

CLIR-ben a gondolat:

```text
load.u16 %0, a
load.u16 %1, b
add.u16 %2, %0, %1
ret %2
```

A `%0`, `%1`, `%2` nem valódi regiszter. Csak ideiglenes, architektúrafüggetlen érték.

## 3. Változó

C-Lite:

```c
u16 x = 10;
x = x + 1;
```

CLIR:

```text
const.u16 %0, 10
store.u16 x, %0
load.u16 %1, x
const.u16 %2, 1
add.u16 %3, %1, %2
store.u16 x, %3
```

Ez mutatja meg a `load` és `store` lényegét.

## 4. `if`

C-Lite:

```c
if (x < 10) {
    x = x + 1;
}
```

CLIR-ben a strukturált `if` címkékre és ugrásra bomlik:

```text
load.u16 %0, x
const.u16 %1, 10
lt.u16 %2, %0, %1
jz %2, if_else_0
...
jmp if_end_1
if_else_0:
if_end_1:
```

Az assembly gondolkodás egyik legfontosabb felismerése: a magas szintű vezérlés végül feltételes és feltétel nélküli ugrás.

## 5. `while`

```c
while (i < n) {
    i = i + 1;
}
```

CLIR:

```text
while_test_0:
  load.u16 %0, i
  load.u16 %1, n
  lt.u16 %2, %0, %1
  jz %2, while_end_1
  ...
  jmp while_test_0
while_end_1:
```

## 6. Tömb

```c
u16 values[4];
values[2] = 100;
```

CLIR fogalmak:

```text
addr %0, values
const.u16 %1, 2
index %2, %0, %1, 2
const.u16 %3, 100
storemem.u16 %2, %3
```

Az `index` utolsó `2` értéke az `u16` elem byte-mérete. `u8` tömbnél ez 1.

## 7. Pointer

```c
u16* p = &values[0];
u16 x = p[1];
```

A pointer C-Lite-ban egyszerű 16 bites memória-cím. A compiler kezeli az elem méretével történő címképzést.

## 8. Függvényhívás

```c
u16 y = add(10, 20);
```

CLIR:

```text
const.u16 %0, 10
const.u16 %1, 20
call.u16 %2 = add(%0, %1)
store.u16 y, %2
```

A CLIR itt sem mondja meg, hogy a target regiszterben, veremben, beltben vagy memóriában adja át a paramétereket.

## 9. MMIO

```c
vstore8(0xff00, 65);
u8 status = vload8(0xff01);
```

CLIR:

```text
storememv.u8 ...
loadmemv.u8 ...
```

A `v` a volatile/MMIO jelleget mutatja. Az ISA konkrét port- vagy memóriautasításait továbbra sem kell megtanulni a C-Lite használatához.

## 10. Ugyanez target assemblyben

Miután a CLIR már érthető:

```sh
svm-clite --target register --emit asm examples/learning_01_arithmetic.cl
svm-clite --target stack --emit asm examples/learning_01_arithmetic.cl
```

Érdemes ugyanazt a forrást két targetre lefordítani és összehasonlítani. A program jelentése azonos, a megvalósítás azonban teljesen más lehet.

## Javasolt sorrend

1. `--check`;
2. változó és aritmetika;
3. `--emit ir`;
4. `if` és `while`;
5. tömb és pointer;
6. függvényhívás;
7. MMIO;
8. csak ezután `--emit asm` két külön architektúrára.

A cél nem az assembly szintaxis memorizálása, hanem annak megértése, hogyan lesz a strukturált programból egyszerű gépi műveletsor.

## A CLIR és a target-kód szétválasztása

A `--emit ir` a nyelvi loweringet mutatja, míg a `--emit asm` az adott architektúra mechanikus leképezését. Nincs optimalizáló réteg. Ugyanaz a CLIR kerül minden targethez; csak az ISA-megvalósítás változik. Lásd `CODEGEN_HU.md`.
