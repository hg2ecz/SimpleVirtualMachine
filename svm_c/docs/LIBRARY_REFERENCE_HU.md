# SVM-C könyvtárkatalógus

A `svm_c/lib/` forráskönyvtárak include-olhatók. `-O1/-O2/-Os` mellett csak a `main()`-ből tranzitívan használt függvények kerülnek a programképbe.

## Umbrella modulok

- `arithmetic.sc` - aritmetikai umbrella include;
- `integer.sc` - integer segédek umbrella;
- `numeric.sc` - numerikus umbrella;
- `float.sc` - soft-float umbrella.

## Egész aritmetika

`arithmetic_int.sc`: `abs`, `min`, `max`, `clamp`, `isqrt`, `powu`, `gcd`, `lcm`.

`signed_int.sc`: i8 sign extension, i16 abs/sign, signed div/mod, compare, arithmetic shift.

`wide_int.sc`: little-endian wordos `u32/i32` tárolás és pointeres add/sub/bitwise/shift/compare/div/mod/negáció; 32x32 -> 64 bites eredmény segédek. A 64 bites típusok elsősorban teljes szorzateredmény és tárolás céljára vannak, nem általános 64 bites ALU-ként.

## Q15 és trigonometria

`q15.sc`: Q15 abs/neg/mul/div.

`trig.sc`: `sin`, `cos`, `tan`; a teljes kör `0..65535`, ahol `0x4000 = 90 fok`. A trigonometria közelítő integer implementáció, nem bitpontos matematikai referencia.

## Soft float

`f16.sc`: IEEE binary16 bitminták 16 bites értékben; add/sub/mul/div, abs/neg, ellenőrzések és u16 konverziók.

`f32.sc`: IEEE binary32 4 byte-os objektumokban, pointeres API-val; add/sub/mul/div, abs/neg, ellenőrzések és konverziók. Nincs hardveres FPU.

## Random

`random.sc`: determinisztikus szoftver PRNG (`rand`, seed/range segédek).

`hrandom.sc`: MMIO PRNG (`hrand`, `hrand_seed`, `hrand_range`). A referencia periféria determinisztikus xorshift32, nem kriptográfiai entrópiaforrás.

## Konzol és képernyő

`console.sc`: `newline`, `puthex16`, `putu16`; a compiler built-in `putc`, `puts`, `getc` függvényeire épül. Lásd [`CONSOLE_LIBRARY_HU.md`](CONSOLE_LIBRARY_HU.md).

`graphics.sc`: 320x200x2 bpp framebuffer rajzolás, 4 palettaslot, pixel/vonal/téglalap/kör. Lásd [`GRAPHICS_LIBRARY_HU.md`](GRAPHICS_LIBRARY_HU.md).

`textscreen.sc`: 40x25, belső 8x8 font-ROM-ra épülő karakteres framebuffer-kezelés, goto/home/CR/LF/cursor/clear. Lásd [`TEXT_SCREEN_LIBRARY_HU.md`](TEXT_SCREEN_LIBRARY_HU.md).

## FFT riport

`fft_report.sc`: az FFT példákhoz használt fixpontos numerikus riport és 32 bites számlálókiírás. Nem általános printf-helyettesítő.
