# Dokumentációs záróaudit – v2.3.24

A cél a v2.3.17–v2.3.23 implementációs állapot teljes dokumentációs lefedése és a komponenshatárok egyértelműsítése.

## Dokumentációs tulajdonosok

- `docs/` – közös hardverplatform, memória/MMIO, videó és ISA;
- `svm_asm/docs/` – assembler CLI, include, architektúránkénti programming manual/instruction reference és assembly helper library;
- `svm_rt/docs/` – executable konténer, runtime futtatás, host I/O, VM struktúra és ciklusmodell;
- `svm_c/docs/` – nyelv, compiler CLI/optimalizálás, ABI/típusok, könyvtárak, smoke tesztek és példák.

## Új vagy teljessé tett referenciaanyagok

### Közös platform

- `MMIO_REFERENCE_HU.md` – teljes `0xFF00..0xFFFF` térkép, hozzáférési szemantika, IRQ, timer, konzol és RNG;
- `VIDEO_TEXT_REFERENCE_HU.md` – külön VRAM, packed 2 bpp formátum, 16 színű master paletta, belső font ROM és 40x25 text layer.

### Runtime

- `RUNTIME_USAGE_HU.md` – host ablak, billentyűzet, stdin/stdout konzol és futási lifecycle;
- `EXECUTABLE_FORMAT_HU.md` – 12 byte-os közös konténer és mind a kilenc magic.

### Assembler

- `ASSEMBLER_REFERENCE_HU.md` – CLI, targetek, include rendszer, memória-konvenció és helper könyvtárak.

### SVM-C

- `COMPILER_REFERENCE_HU.md` – CLI, targetek, `--emit`, optimalizációs szintek és unused-function elimination;
- `LIBRARY_REFERENCE_HU.md` – teljes `svm_c/lib` moduljegyzék;
- `SMOKE_TESTS_HU.md` – részletes és compact/full numerikus regressziós rendszer.

## Külön rögzített aktuális döntések

- CPU RAM: `0x0000..0xFEFF`, egybefüggő;
- MMIO: `0xFF00..0xFFFF`;
- VRAM: külön 16 KiB, framebuffer `0x0000..0x3E7F`;
- nincs guest-visible System ROM;
- Stack: kétcellás TOS/NOS lazy cache, ISA-változás nélkül;
- Load/Store és Register-Memory: `R6` az egységes stack pointer;
- Memory-to-Memory: `A3` az egységes stack pointer;
- `-O1/-O2/-Os`: `main()`-ből tranzitívan elérhetetlen függvények kiesnek még layout előtt;
- `-O0` és `svm-c-unopt-only`: minden parsed függvényt megtart;
- hardveres floating point nincs;
- `f16/f32` szoftveres;
- `u64/i64` nem általános 64 bites ALU, főleg storage és teljes 32x32 szorzateredmény;
- a 2 bpp pixel 0..3 palettaslot, a slot 0..15 master-paletta indexre mutat;
- a karakteres képernyő a framebufferbe rajzoló 8x8 font periféria, nem külön text RAM;
- VT100/RS-232 konzol és framebuffer text layer külön interfész.

## Ellenőrzések

- Markdown relatív linkek fájlrendszer-szintű ellenőrzése;
- régi `0x7000`, `0x6BFF`, CPU-visible VRAM és System ROM állítások keresése;
- MMIO dokumentáció összevetése a `svm_rt/src/memory.rs` konstansaival;
- executable magic táblázat összevetése a kilenc assembler `program.rs` fájljával;
- könyvtár API-k összevetése a `svm_c/lib/*.sc` publikus függvényeivel.

A történeti review dokumentumokban előforduló korábbi cím vagy döntés történeti kontextusnak számít; az aktuális normatív referencia a komponens README-kből elérhető jelenlegi platform/ISA/compiler/runtime dokumentáció.
