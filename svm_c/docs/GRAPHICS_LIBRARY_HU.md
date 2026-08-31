# SVM-C 2 bpp grafikai segédkönyvtár

Include:

```c
include "lib/graphics.sc";
```

A közös videó 320x200 pixel, 2 bpp, külön 16 KiB VRAM-mal. Négy pixel fér egy byte-ba; a bal szélső pixel a 7..6 biten van.

## Színmodell

A pixelérték 0..3 közötti palettaslot. A `0xFF0C..0xFF0F` MMIO-regiszterek a négy slotot a fix 16 színű RGBI/EGA-szerű master paletta indexeihez rendelik.

```c
gfx_set_palette(0, 9, 11, 15);
clear(0);
line(10, 309, 10, 189, 3);
```

## API

- `gfx_set_palette(p0,p1,p2,p3)` - palettaslotok master-paletta indexei;
- `gfx_default_palette()` - fekete, sötétszürke, világosszürke, fehér;
- `gfx_set_color(color)` / `gfx_get_color()` - kényelmi aktuális szín;
- `putpixel(x,y,color)` - explicit színű pixel, képernyőn kívül nem ír;
- `putpixel_current(x,y)` - pixel az aktuális színnel;
- `getpixel(x,y)` - framebuffer-slot 0..3;
- `clear(color)` - teljes framebuffer kitöltése;
- `hline(x0,x1,y,color)` - vízszintes vonal;
- `vline(x,y0,y1,color)` - függőleges vonal;
- `line(x1,x2,y1,y2,color)` - egész DDA-vonal képernyő-koordinátákhoz;
- `rect(x,y,w,h,color)` - téglalap kerete;
- `fillrect(x,y,w,h,color)` - kitöltött téglalap;
- `circle(cx,cy,r,color)` - egész oktáns-szkennelésű körvonal;
- `fillcircle(cx,cy,r,color)` - kitöltött kör vízszintes spanekkel;
- `line_current`, `rect_current`, `fillrect_current`, `circle_current`, `fillcircle_current` - az aktuális színt használó kényelmi wrapperek.

A geometriai API elsődlegesen explicit színt kap, így egy hívás önmagában leírja a rajzolási műveletet, és nem függ rejtett globális állapottól. A `line` végpontjai képernyő-koordináták legyenek; a `circle/fillcircle` esetén a teljes körnek a framebufferen belül kell lennie. Ez ugyanaz a tartományfeltétel, mint a kézi Assembly magas szintű rutinjainál.

## Több mint négy paraméter

Az SVM-C-ben már nincs négyelemű paraméterkorlát. A paraméterek statikusan kiosztott callee-helyeken élnek. A Register/Accumulator/MemReg backendcsalád előbb a runtime stacken biztonságosan stagingeli az összes argumentumértéket, majd a `CALL` előtt a callee paraméterhelyeire írja őket; a Stack backend a saját természetes vermes paraméterátadását használja. Emiatt a `line(...,color)` 5 paraméteres hívása, illetve későbbi 6-7 paraméteres primitívek is ABI-váltás nélkül támogatottak.

Az ABI továbbra sem rekurzív és nem reentráns, mert a lokálisok és paraméterek statikus tárolásúak.

## Kódméret

Optimalizált C fordításnál a C-szintű unused-function elimination már kódgenerálás előtt eltávolíthat elérhetetlen függvényeket. Bináris készítéskor az assembler procedure-GC ezen felül minden optimalizálási szinten kiszedi az elérhetetlen `.proc` blokkokat és runtime/library függőségeket.

## Háromszögek

A C grafikai könyvtár további primitívjei:

```c
triangle(x1, y1, x2, y2, x3, y3, color);
filltriangle(x1, y1, x2, y2, x3, y3, color);
```

A kitöltött változat scanline kitöltést használ. A hordozható algoritmus C-ben kanonikus; nem szükséges mind a kilenc ISA assembly könyvtárában külön kézzel fenntartani.
