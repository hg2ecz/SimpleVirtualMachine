# Architektúra-specifikus matematikai ASM demók

A hordozható algoritmusok elsődleges implementációja továbbra is az `svm_c/lib` C könyvtár.
Az `svm_asm/lib/<target>/math.asm` fájlok oktatási és ISA-demonstrációs célúak: ugyanazokat az alapfeladatokat mutatják meg a kilenc virtuális architektúra természetes operandusmodelljével.

Minden target könyvtárában megtalálható:

- `math.asm` – natív integer aritmetika és f16/f32 bitreprezentációs segédek, amennyire az adott ISA demójához célszerű;
- `convert.asm` – típuskonverziós demó belépési pont;
- `typed_arith.asm` – a `math.asm` név szerinti include-ja;
- `typed_convert.asm` – a `convert.asm` név szerinti include-ja;
- `examples/<target>/typed_math_demo.asm` – procedure-GC kompatibilis használati példa.

A teljes decimális/bináris/hexadecimális string parse+format referencia jelenleg a Register ASM könyvtárban (`typed_convert.asm`) és a hordozható C könyvtárban (`svm_c/lib/convert.sc`) található. A többi ISA `convert.asm` fájlja demonstrációs réteg; ezek bővítésekor a C implementáció a viselkedési referencia.

Az IEEE-754 `f16`/`f32` teljes szoftveres aritmetikájának kanonikus megvalósítása továbbra is C-ben van (`f16.sc`, `f32.sc`). Az ASM demók elsősorban a reprezentációs és integer-primitíveket mutatják, mert kilenc teljes soft-float implementáció párhuzamos karbantartása nem cél.
