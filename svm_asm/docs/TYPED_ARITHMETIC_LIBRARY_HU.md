# Típusos assembly aritmetikai mintakönyvtár

A projekt algoritmikus referencia-implementációja továbbra is SVM-C. Az assembly oldal célja az ABI, a többwordös műveletek és a kézi optimalizálás bemutatása, ezért a teljes mintakészlet elsődlegesen a Register ISA alatt található:

- `lib/register/typed_arith.asm`
- `lib/register/typed_convert.asm`

## Egészek

A könyvtár tartalmaz `u8/i8/u16/i16` add/sub/mul/div jellegű rutinokat, továbbá `u32/i32` add/sub/mul (modulo 2^32) primitíveket. A 32 bites ABI: `R1:R0` az első, `R3:R2` a második operandus és `R1:R0` az eredmény.

A 8 bites műveletek explicit maszkolást/sign-extensiont végeznek, ezért a 16 bites CPU-n is a deklarált típus szemantikáját demonstrálják.

## Lebegőpontos típusok

`f16` = IEEE-754 binary16 bitminta egy 16 bites szóban, `f32` = IEEE-754 binary32 két 16 bites szóban. A CPU-kban nincs FPU. A teljes soft-float `f16_add/sub/mul/div` és `f32_add/sub/mul/div` kanonikus implementációja az SVM-C `lib/f16.sc` és `lib/f32.sc` fájljaiban marad. Az assembly mintakönyvtár representation helper-eket és a soft-float számára szükséges multiword integer primitíveket mutatja; így nem tartunk fenn kilenc, kézzel duplikált IEEE implementációt.

## Szövegkonverzió

A `typed_convert.asm` NUL-terminált ASCII bufferrel dolgozik. Közvetlenül elérhető például:

- `u8_to_decstr`, `i8_to_decstr`
- `u16_to_decstr`, `i16_to_decstr`
- `u16_to_hexstr`, `u16_to_binstr`
- `parse_u16_decstr`, `parse_u16_hexstr`, `parse_u16_binstr`
- `f16_to_hexstr`, `parse_f16_hexstr`
- `f32_to_hexstr`

A `f16/f32` hex forma a pontos IEEE bitmintát írja/olvassa. A valódi lebegőpontos decimális szövegkonverzió továbbra is C-ben célszerű, mert az már önmagában jelentős soft-float algoritmus.

A procedure-GC miatt egy teljes include sem építi be a nem használt rutinokat.
