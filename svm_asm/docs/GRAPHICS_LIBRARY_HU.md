# Assembly 2 bpp grafikai segédkönyvtár

A közös framebuffer 320x200, 2 bpp, külön VRAM-ban. A pixelérték 0..3 palettaslot. A `0xFF0C..0xFF0F` palettaregiszterek rendelik a négy slotot a 0..15 master-paletta indexekhez. A master-paletta színsorrendje: fekete, kék, zöld, cián, vörös, bíbor, barna, világosszürke, sötétszürke, világoskék, világoszöld, világos cián, világos vörös, világos bíbor, sárga, fehér.

Architektúránként include-olható fájl:

`svm_asm/lib/<arch>/graphics.asm`

Az assembly oldalon a `register/graphics.asm` a teljes kézzel írt packed-pixel referencia: `gfx_set_palette`, `gfx_set_color`, `putpixel`, `clear`, `hline`, `vline`. A többi ISA `graphics.asm` fájlja a közös színmodellt, scratch-konvenciót és a közvetlen VRAM elérést rögzíti; a magasabb szintű, minden targeten azonos `line`, `rect`, `fillrect`, `circle` implementáció az SVM-C könyvtárban található.

Ez szándékos: a kilenc ISA-n a kézi assembly hívási ABI eltér, miközben az SVM-C könyvtár ugyanabból a forrásból mind a kilenc célra lefordul.

A helper könyvtárak scratch címe `0x00E8..0x00EF`; assembly program használatakor ezt a 8 byte-ot ne használja más célra.
