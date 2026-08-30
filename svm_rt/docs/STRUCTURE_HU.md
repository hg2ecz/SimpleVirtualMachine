# Egységes gépstruktúra

## Alapelv

A gép környezete közös, csak a CPU végrehajtási elve tér el. A kilenc CPU ugyanazt a 64 KiB-os rendszercímtérképet, MMIO-t, külön 16 KiB VRAM-ot, videóvezérlőt, palettát, konzolt, timer/IRQ modellt és ciklusszámlálást használja.

## Nincs System ROM

A korábbi 8 KiB-os `0xE000..0xFFFF` System ROM megszűnt. Ennek oka, hogy a firmware-rutinok CPU-specifikus gépi kódot tartalmaztak, ezért valójában négy külön ROM-kép volt. A szolgáltatások túl kicsik ahhoz, hogy ez indokolt legyen.

- a VT100 `putc/getc` közvetlen MMIO műveletekre fordul;
- a kurzorpozíció közvetlen `TEXT_X/TEXT_Y` MMIO;
- a karakter kirajzolását a `TEXT_CHAR` periféria végzi;
- a 8×8 font a videóeszköz belső, csak olvasható karakter-ROM-ja, nem része a CPU címtérnek.

Ennek eredménye: nincs CPU-specifikus firmware és nincs ROM ABI. A korábbi ROM-terület `0xE000..0xFEFF` része normál RAM; csak a legfelső `0xFF00..0xFFFF` lap MMIO. A runtime ezen belül a `0xFB00..0xFEFF` RAM-részt stack-konvencióra tartja fenn.

A CPU fizikailag összefüggő RAM-ja `0x0000..0xFEFF`. Az SVM-C statikus allocator a zero-page (`0x0000..0x00EF`) kimerülése után a felső RAM `0xE000..0xFAFF` részét használja. A C binárisnál ezért a compiler ellenőrzi, hogy a `0x0100`-tól növő programkép ne érje el a `0xE000` határt. Assembly programok számára a teljes RAM címezhető; a `0xE000` határ csak az SVM-C jelenlegi statikus-ABI konvenciója.

## Runtime szerkezet

`memory.rs`, `video.rs`, `program.rs`, `machine.rs`, `error.rs` közös. A `cpu/` könyvtárban kilenc CPU-mag van: register, stack, accumulator, memreg, loadstore, regmem, memory2memory, belt, tta. A futtató az executable magic alapján automatikusan kiválasztja a megfelelő CPU-t.

## Assembler

Egy `svm-asm` crate tartalmazza mind a kilenc ISA modult. A CLI célarchitektúrát kap, például `svm-asm register ...`, `svm-asm loadstore ...`, `svm-asm regmem ...`, `svm-asm memory2memory ...`.


## Újonnan implementált architektúrák

A Load/Store, Register-Memory és Memory-to-Memory CPU már része a runtime/assembler/C compiler CLI-nak. A közös platformprofil változatlan maradt; csak az operandusmodell és a CPU decode/végrehajtási logika tér el. Részletek: `../../docs/IMPLEMENTATION_STATUS_HU.md` és `../../docs/ARCHITECTURE_DESIGN_RATIONALE_HU.md`.

A `belt` target az implicit-result CPU. Saját assembler modulja `svm_asm/src/belt/`, runtime magja `svm_rt/src/cpu/belt.rs`, C backendje `svm_c/src/backend/belt.rs`. A backend első változata a virtuális C temporaries-t compiler-owned memóriába süllyeszti, majd valódi belt-műveleteket generál.

A `tta` target a kilencedik, transport-triggered CPU. A TTA16 moduljai: `svm_asm/src/tta/`, `svm_rt/src/cpu/tta.rs`, `svm_c/src/backend/tta.rs`; executable formátuma `.svt`, magicje `SVT\x01`.
