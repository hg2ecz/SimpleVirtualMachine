# Közös SVM platform

A kilenc CPU ugyanazt a gépkörnyezetet használja. Csak az utasításkészlet, regiszter-/veremmodell és végrehajtási elv tér el.

## CPU-címtér

- 16 bites címzés, `0x0000..0xFFFF`.
- A CPU-címtér `0x0000..0xFEFF` tartománya összefüggő RAM. Az MMIO a legfelső 256 bájtos `0xFF00..0xFFFF` lapon van; a jelenleg definiált regiszterek `0xFF00..0xFF2A` között találhatók.
- Nincs guest-visible System ROM.
- A korábbi System ROM megszűnt. A `0xE000..0xFEFF` tartomány normál RAM; a legfelső `0xFF00..0xFFFF` lap az új MMIO-hely.
- A CPU csak ebből a címtérből fetch-el utasítást.

## Video RAM

A VRAM külön adatcímtér, 16 KiB (`0x0000..0x3FFF`). Utasítás-fetch nincs belőle.

- 320×200, 2 bpp
- framebuffer: 16 000 byte (`0x0000..0x3E7F`)
- maradék: 384 byte fenntartott (`0x3E80..0x3FFF`)
- 4 egyidejű szín; a négy slot egy fix 16 színű master palettából választható

## Karaktergenerátor

A 8×8 ASCII font 768 byte-os, belső karakter-ROM a közös videóeszközben. Nem része a CPU címtérnek. A `TEXT_CHAR` (`0xFF06`) írásakor a videóeszköz a glyph-et közvetlenül a framebufferbe rajzolja. Emiatt nincs szükség CPU-specifikus font-/firmware-ROM-ra.

## Konzol

- `0xFF20`: VT100/RS232 DATA
- `0xFF21`: STATUS

A C `putc/getc/puts` közvetlen MMIO-kódot generál, nem firmware-hívást. Assemblyből ugyanez közvetlen memória-I/O műveletekkel érhető el.

## Következmény

A platform implementációja egyetlen közös `memory.rs` + `video.rs`; a kilenc CPU-mag csak a saját végrehajtási logikáját tartalmazza.

## Hardverrel segített véletlenszám-generátor

A közös platform egy kis 16 bites hardver-PRNG perifériát tartalmaz. Ez nem CPU-utasítás, ezért mind a kilenc architektúrán azonos költségű és azonos felületű MMIO-eszköz.

- `0xFF26`: `RNG_DATA_LO` — olvasása előállít és latch-el egy új 16 bites mintát
- `0xFF27`: `RNG_DATA_HI` — az utoljára latch-elt minta felső byte-ja
- `0xFF28`: `RNG_STATUS` — bit 0 (`RNG_READY`) jelenleg mindig 1
- `0xFF29..0xFF2A`: `RNG_SEED` — írható 16 bites seed

A normál 16 bites olvasás (`load16(0xFF26)`) egy összetartozó 16 bites mintát ad. A jelenlegi VM referenciaimplementáció egy kis, determinisztikus `xorshift32` állapotgépet használ. Ez olcsó hardver-PRNG modell és reprodukálható tesztforrás, **de nem valódi entrópiaforrás és nem kriptográfiai RNG**. Fizikai implementációban ugyanaz az MMIO interfész valódi zaj-/jitteralapú entrópiára köthető; emulátorban host-OS entrópia használható, ha nemdeterminisztikus működés szükséges.

A C könyvtári felület a `hrandom.sc` modulban található: `hrand()`, `hrand_max()`, `hrand_seed()` és `hrand_range()`.


## Integer aritmetikai segédletek

A többwordös egész és soft-float könyvtárakat kis költségű integer segédutasítások támogatják; hardveres floating point nincs. Lásd: `ARCHITECTURE_DESIGN_RATIONALE_HU.md`.

### Belt16 belső stack-területek

A RAM fizikailag nem darabolódik stack-szigetekre. A runtime konvenció szerint a felső 1 KiB RAM (`0xFB00..0xFEFF`) a futó CPU stackterülete. A Stack, Belt16 és TTA16 ezt két 512 bájtos részre osztja: data stack `0xFB00..0xFCFF`, control/return stack `0xFD00..0xFEFF`. Ezek továbbra is közönséges RAM-címek, nem külön címtér.


### Veremmutató-konvenció

A Load/Store és Register-Memory CPU-n az `R6`, a Memory-to-Memory CPU-n az `A3` az architekturálisan látható veremmutató. A compiler ideiglenes `PUSH/POP` műveletei, valamint a `CALL/RET` és IRQ mentések ugyanazt a veremet használják; nincs külön rejtett második SP.
