# Videó, framebuffer, paletta és karakteres réteg

## Külön VRAM

A videómemória 16 KiB-os külön adatcímtér (`0x0000..0x3FFF`). A CPU nem fetch-elhet innen utasítást.

- felbontás: 320x200 pixel;
- színmélység: 2 bpp;
- framebuffer: 16 000 byte (`0x0000..0x3E7F`);
- fenntartott videóterület: 384 byte (`0x3E80..0x3FFF`).

Négy pixel fér egy byte-ba. Balról jobbra a bitek: `7..6`, `5..4`, `3..2`, `1..0`.

## Paletta

Minden 2 bites pixelérték egy 0..3 palettaslot. A `VIDEO_PALETTE0..3` MMIO-regiszterek (`0xFF0C..0xFF0F`) választják ki, hogy az adott slot a fix 16 színű master paletta melyik színét jelentse.

| Index | Szín | RGB |
|---:|---|---|
| 0 | fekete | `#000000` |
| 1 | kék | `#0000AA` |
| 2 | zöld | `#00AA00` |
| 3 | cián | `#00AAAA` |
| 4 | vörös | `#AA0000` |
| 5 | bíbor | `#AA00AA` |
| 6 | barna/sötét sárga | `#AA5500` |
| 7 | világosszürke | `#AAAAAA` |
| 8 | sötétszürke | `#555555` |
| 9 | világoskék | `#5555FF` |
| 10 | világoszöld | `#55FF55` |
| 11 | világos cián | `#55FFFF` |
| 12 | világos vörös | `#FF5555` |
| 13 | világos bíbor | `#FF55FF` |
| 14 | sárga | `#FFFF55` |
| 15 | fehér | `#FFFFFF` |

Alapértelmezett slotok: `0 -> 0`, `1 -> 8`, `2 -> 7`, `3 -> 15`.

## Belső 8x8 font ROM

A videóeszköz 96 glyph-et tartalmaz `0x20..0x7F` karakterkódokra. Egy glyph 8 byte magas, összesen 768 byte. A font-ROM nem CPU- és nem VRAM-címezhető; kizárólag a `TEXT_CHAR` periféria használja.

## 40x25 karakteres framebuffer-réteg

A 8x8 font miatt a 320x200 framebuffer logikailag 40x25 karaktercellára osztható. A karakteres MMIO nem külön text RAM: `TEXT_CHAR` írásakor a videóeszköz azonnal pixeleket rajzol a framebufferbe az aktuális foreground/background slotokkal.

Regiszterek:

- `TEXT_X` `0xFF02`
- `TEXT_Y` `0xFF03`
- `TEXT_FG` `0xFF04`
- `TEXT_BG` `0xFF05`
- `TEXT_CHAR` `0xFF06`

A magasabb szintű C API-t lásd: [`../svm_c/docs/TEXT_SCREEN_LIBRARY_HU.md`](../svm_c/docs/TEXT_SCREEN_LIBRARY_HU.md). Assembly API: [`../svm_asm/docs/TEXT_SCREEN_LIBRARY_HU.md`](../svm_asm/docs/TEXT_SCREEN_LIBRARY_HU.md).

## Grafikai könyvtárak

C-ben: [`../svm_c/docs/GRAPHICS_LIBRARY_HU.md`](../svm_c/docs/GRAPHICS_LIBRARY_HU.md). Assemblyben: [`../svm_asm/docs/GRAPHICS_LIBRARY_HU.md`](../svm_asm/docs/GRAPHICS_LIBRARY_HU.md).
