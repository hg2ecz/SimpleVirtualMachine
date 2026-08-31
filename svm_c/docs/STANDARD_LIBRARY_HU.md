# SVM-C általános könyvtár

Az SVM-C általános algoritmuskönyvtára elsődlegesen **C forrásban** készül. Ennek oka, hogy a projekt fő programozási nyelve C, ezért az algoritmusok egyetlen hordozható implementációból használhatók mind a kilenc SVM targeten. Kézzel írt assembly elsősorban alacsony szintű ABI/MMIO helperhez, célzott optimalizációhoz és demonstrációhoz marad.

A modulok a `svm_c/lib/` könyvtárban vannak, és például így húzhatók be:

```c
include "memory.sc";
include "crc.sc";
```

vagy az általános gyűjtőfájllal:

```c
include "stdlib.sc";
```

Fordításkor a könyvtárat `-I svm_c/lib` kapcsolóval lehet felvenni. `-O1/-O2/-Os` esetén a C DCE, bináris készítéskor pedig az assembler procedure-GC gondoskodik arról, hogy a nem használt függvények ne kerüljenek a programképbe.

## `memory.sc`

Byte-címzésű memóriafüggvények, ahol a címeket `u16` értékek képviselik:

- `mem_zero(dst, count)`
- `memset(dst, value, count)`
- `memcpy(dst, src, count)`
- `memmove(dst, src, count)`
- `memcmp(a, b, count)`

A `memmove` átfedő tartományokat is kezel. A `memcmp` 0-t ad egyenlőségre, `0xffff` értéket `a<b`, és 1-et `a>b` esetén.

## `string.sc`

Nullával lezárt byte-stringekhez:

- `strlen(s)`
- `strcmp(a, b)`
- `strncmp(a, b, count)`
- `strcpy(dst, src)`
- `strncpy(dst, src, count)`
- `strchr(s, ch)`
- `streq(a, b)`

A string-cím `u16`. Tömb címét például `&buffer` alakban lehet átadni.

## `bits.sc`

16 bites bitműveletek:

- `rotl16`, `rotr16`
- `popcount16`
- `parity16`
- `clz16`, `ctz16`
- `bitreverse16`
- `bswap16`

Ezek hordozható C referencia-implementációk. Ha valamely target később natív utasítást kap, a backend intrinsic-ként optimalizálhatja ugyanazt a műveletet.

## `crc.sc`

Kommunikációs és tárolási ellenőrzőösszegek:

- `checksum8(data, count)`
- `checksum16(data, count)`
- `crc8_update(crc, byte)`
- `crc8(data, count)` — CRC-8/ATM, poly `0x07`, init `0x00`
- `crc16_ccitt_update(crc, byte)`
- `crc16_ccitt(data, count)` — CRC-16/CCITT-FALSE, poly `0x1021`, init `0xffff`

Az `_update` változatok streamelt feldolgozáshoz használhatók, például soros kommunikációnál.

## `convert.sc`

Szám/string konverzió:

- `digit_value(ch)`
- `parse_u16_dec(s)`
- `parse_i16_dec(s)`
- `parse_hex16(s)`
- `u16_to_hex(dst, value)` — 4 hex számjegy + NUL
- `u16_to_dec(dst, value)` — max. 5 számjegy + NUL
- `i16_to_dec(dst, value)` — előjel + max. 5 számjegy + NUL

A parse rutinok az első nem megfelelő karakternél megállnak. Az első változat overflow-ellenőrzést nem végez; az aritmetika 16 biten körbefordul.

## `buffer.sc`

Egyszerű byte ring buffer statikus memória használatához:

- `ring_init(head_addr, tail_addr)`
- `ring_empty(head_addr, tail_addr)`
- `ring_count(capacity, head_addr, tail_addr)`
- `ring_full(capacity, head_addr, tail_addr)`
- `ring_push(data, capacity, head_addr, tail_addr, value)`
- `ring_pop(data, capacity, head_addr, tail_addr, out_addr)`

A megoldás egy üres slotot fenntart, így az üres és tele állapot külön számláló nélkül megkülönböztethető. `capacity >= 2` szükséges.

## `console.sc` új cím-alapú rutinjai

A compiler built-in `puts()` csak string literált fogad. Futás közben létrehozott bufferhez ezért:

- `putstr(address)` — NUL-terminált memóriastring kiírása
- `puti16(value)` — előjeles decimális kiírás
- `putbin16(value)` — 16 bites bináris kiírás

A meglévő `putu16`, `puthex16`, `newline` továbbra is használható.

## `stdlib.sc`

Az általános umbrella include jelenleg behúzza:

- integer/signed/wide aritmetikát;
- Q15 és trigonometriai rutint;
- memória- és stringfüggvényeket;
- bitműveleteket;
- CRC/checksumot;
- konverziót;
- ring buffert;
- szoftveres PRNG-t;
- konzol segédeket.

A grafika, textscreen, hardveres random és soft-float szándékosan külön opt-in modul marad.

## Assembly szerepe

A hordozható algoritmusok kanonikus implementációja C. Assemblyben nem célszerű ugyanazt kilencszer kézzel karbantartani. A `svm_asm/lib/register/algorithms_demo.asm` és a hozzá tartozó példa bemutatja, hogyan írható célzott kézi helper, de ez demonstráció, nem külön párhuzamos standard library.
