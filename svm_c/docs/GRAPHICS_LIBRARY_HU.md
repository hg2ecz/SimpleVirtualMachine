# SVM-C 2 bpp grafikai segédkönyvtár

Include:

```c
include "lib/graphics.sc";
```

A közös videó 320x200 pixel, 2 bpp, külön 16 KiB VRAM-mal. Négy pixel fér egy byte-ba; a bal szélső pixel a 7..6 biten van.

## Színmodell

A pixelérték **0..3**, és egy palettaslotot választ. A slot nem közvetlen RGB-szín. A négy slothoz a `0xFF0C..0xFF0F` MMIO-regiszterek rendelnek egy-egy indexet a fix 16 színű RGBI/EGA-szerű master palettából.

| master index | szín |
|---:|---|
| 0 | fekete |
| 1 | kék |
| 2 | zöld |
| 3 | cián |
| 4 | vörös |
| 5 | bíbor |
| 6 | barna/sötét sárga |
| 7 | világosszürke |
| 8 | sötétszürke |
| 9 | világoskék |
| 10 | világoszöld |
| 11 | világos cián |
| 12 | világos vörös |
| 13 | világos bíbor |
| 14 | sárga |
| 15 | fehér |

Példa:

```c
gfx_set_palette(0, 9, 11, 15); // slot0 fekete, slot1 világoskék, slot2 világos cián, slot3 fehér
gfx_set_color(3);               // ezután a rajzolás fehérrel történik
```

## API

- `gfx_set_palette(p0,p1,p2,p3)` - a négy framebuffer-slot master-paletta indexe, 0..15.
- `gfx_default_palette()` - fekete, sötétszürke, világosszürke, fehér.
- `gfx_set_color(color)` / `gfx_get_color()` - aktuális rajzszín, 0..3.
- `putpixel(x,y)` - pixel az aktuális színnel; képernyőn kívül nem ír.
- `putpixelc(x,y,color)` - pixel explicit 0..3 színnel.
- `getpixel(x,y)` - framebuffer-slot 0..3.
- `clear(color)` - teljes képernyő törlése egy slotértékkel.
- `hline(x0,x1,y)`, `vline(x,y0,y1)` - vízszintes/függőleges vonal.
- `line(x0,y0,x1,y1)` - Bresenham-vonal, az aktuális színnel.
- `rect(x,y,w,h)` - téglalap kerete.
- `fillrect(x,y,w,h)` - kitöltött téglalap.
- `circle(cx,cy,r)` - midpoint kör.

A `line`, `rect`, `fillrect` és `circle` azért az aktuális színt használja, mert az SVM-C ABI legfeljebb négy függvényparamétert enged.

`-O1/-O2/-Os` esetén az unused-function elimination miatt a `graphics.sc` nem húzza be automatikusan az összes fenti rutint: csak a `main()`-ből tranzitívan elérhető függvények kapnak gépi kódot.
