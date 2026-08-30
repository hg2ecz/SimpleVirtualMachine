# SVM-C numerikus típusok és soft-aritmetika

## Típusok

A nyelv tárolási típusai:

| Típus | Méret | Megjegyzés |
|---|---:|---|
| `bool` | 1 byte | 0 = hamis, nem nulla = igaz |
| `i8` | 1 byte | kétkomplementes bitminta |
| `u8` | 1 byte | unsigned |
| `i16` / `int` | 2 byte | kétkomplementes bitminta |
| `u16` | 2 byte | unsigned |
| `i32` / `long` | 4 byte | többwordös, cím-alapú objektum |
| `u32` | 4 byte | többwordös, cím-alapú objektum |
| `i64` | 8 byte | csak 32x32 signed szorzat teljes eredményének tárolására |
| `u64` | 8 byte | csak 32x32 unsigned szorzat teljes eredményének tárolására |

A CPU-k továbbra is 16 bites integer gépek. A `bool/i8/u8/i16/u16` natív skalárként fordul. A 32 és 64 bites objektumoknál nincs rejtett 32/64 bites virtuális ALU: a könyvtári rutinok címeket kapnak.

A nyelv ezért támogatja a név szerinti címképzést:

```c
u32 a;
u32 b;
u32 c;
u32_add(&c, &a, &b);
```

A `u32/i32/i64/u64` közvetlen értékkifejezésként, by-value paraméterként vagy függvény-visszatérési típusként szándékosan nem használható. Ez a korlátozás tartja kicsiben és összehasonlíthatóban mind a kilenc ISA backendjét.

## Signed 8/16 bit

A bitenkénti, összeadásos, kivonásos és szorzásos kétkomplementes műveletek a natív 16 bites ALU-val végezhetők. A signed-érzékeny műveletekhez a `signed_int.sc` ad segédfüggvényeket, például `i8_sext`, `i16_div`, `i16_mod`, `i16_lt`, `i16_asr1`.

## 32 bites aritmetika

A `wide_int.sc` pointeres API-t ad:

```c
u32_from_u16(&a, 1234);
u32_add(&c, &a, &b);
u32_sub(&c, &a, &b);
u32_div(&c, &a, &b);
u32_mod(&c, &a, &b);
u32_and(&c, &a, &b);
u32_or(&c, &a, &b);
u32_xor(&c, &a, &b);
u32_shl1(&c, &a);
u32_shr1(&c, &a);
```

A teljes szélességű szorzat:

```c
u64 p;
u32_mul_u64(&p, &a, &b);
```

Signed változat:

```c
i64 p;
i32_mul_i64(&p, &a, &b);
```

`i64/u64` számára nincs publikus add/sub/div/mod/shift API. A library belső `__u64_*` segédrutinjai kizárólag a teljes 32x32 szorzat előállításához szükségesek.

## f16

Az `f16.sc` IEEE-754 binary16 bitmintát használ `u16`-ban:

```c
u16 a;
u16 b;
a = f16_from_u16(3);
b = f16_from_u16(4);
a = f16_add(a,b);
```

Elérhető: `f16_add`, `f16_sub`, `f16_mul`, `f16_div`, `f16_neg`, `f16_abs`, osztályozás és `u16` konverzió.

## f32

Az `f32.sc` IEEE-754 binary32 bitmintát tárol `u32` objektumban:

```c
u32 a;
u32 b;
u32 c;
f32_from_u16(&a,3);
f32_from_u16(&b,4);
f32_add(&c,&a,&b);
```

Elérhető: `f32_add`, `f32_sub`, `f32_mul`, `f32_div`, `f32_neg`, `f32_abs`, osztályozás és `u16` konverzió.

Az első soft-float implementáció NaN/Inf/zero kódokat kezeli, de a mantissza kerekítése még egyszerű truncation; teljes round-to-nearest-even guard/round/sticky kezelés későbbi pontossági fejlesztés lehet. A CPU ISA-khoz ehhez nem kerül float utasítás.

## Include

A teljes készlet:

```c
include "arithmetic.sc";
```

vagy azonos tartalommal:

```c
include "numeric.sc";
```
