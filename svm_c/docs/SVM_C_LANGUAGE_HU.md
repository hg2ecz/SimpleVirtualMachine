# SVM-C nyelvi áttekintés

Az SVM-C tudatosan kicsi, C-szerű freestanding rendszerprogramozási nyelv a kilenc SVM CPU-hoz. Nem ANSI/ISO C.

A jelenlegi nyelv támogatja a `bool`, `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `int`, `long`, `void` típusokat, statikus változókat és fix tömböket, `[]` indexelést, függvényeket, `if/else`, `while`, `for`, `do...while`, `break`, `continue`, rövidzáras `&&/||` operátorokat, statement-szintű `++/--` és compound assignment műveleteket, korlátozott `sizeof`-ot és `puts("...")` VT100 kiírást.

A lokális változók és paraméterek nem stack frame-ben, hanem statikusan kiosztott memóriában vannak. A hívó a callee paraméterhelyeire írja az argumentumokat; ezért a korábbi négyelemű paraméterkorlát megszűnt. Ez kis compiler/backend költséget ad, de nincs rekurzió és reentrancia.

A teljes, implementációval szinkronban tartott referencia: **`C_REFERENCE_HU.md`**; a tudatosan kihagyott ANSI C elemek és a nyelvi határok ebben a referenciában szerepelnek.


## Forrás include

Saját könyvtárak behúzásához a fordító a `include "fajl.sc";` formát támogatja. Ez nem preprocesszor: a fájl szövege a fordítás előtt ugyanabba a fordítási egységbe kerül. Az útvonal a behúzó fájlhoz képest relatív, illetve `-I` keresési könyvtárak adhatók meg. Egy fájl fordításonként egyszer kerül be.


## Széles numerikus objektumok

Az `i32/u32` 4 bájtos, az `i64/u64` 8 bájtos tárolási objektum. A 32/64 bites értékek könyvtári, cím-alapú soft-aritmetikával használhatók; a `&objektum` címképzés támogatott. `int` = `i16`, `long` = `i32`. Az `i64/u64` publikus szerepe a 32×32 bites teljes szorzat eredményének tárolása. Részletek: `NUMERIC_TYPES_HU.md`.
