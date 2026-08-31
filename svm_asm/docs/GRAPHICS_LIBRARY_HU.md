# Assembly grafikai könyvtár

A közös 320x200 képpontos, 2 bpp SVM framebufferhez minden assembly target saját implementációt kap:

`svm_asm/lib/<arch>/graphics.asm`

A teljes fájl biztonságosan include-olható, mert minden rutin `.proc/.endproc` blokkban van, és a procedure-GC csak az elérhető eljárásokat tartja meg.

## Alapprimitívek

Minden target exportálja:

- `gfx_set_color` - aktuális 0..3 palettahely;
- `gfx_set_palette` - a négy palettahely beállítása;
- `putpixel` - egy pixel az aktuális színnel;
- `clear` - teljes framebuffer kitöltése;
- `hline` - vízszintes vonal;
- `vline` - függőleges vonal.

Ezek ABI-ja az adott ISA természetes adatmozgatási modelljét követi. A pontos regiszter/verem/címregiszter kiosztás az adott `graphics.asm` fejlécében és az architektúra Assembly Programming Manualjában található.

## Magasabb szintű geometria

Minden targeten ugyanazok a magasabb szintű rutinok is rendelkezésre állnak:

- `line` - tetszőleges irányú egész koordinátás vonal;
- `rect` - téglalap kerete;
- `fillrect` - kitöltött téglalap;
- `circle` - körvonal;
- `fillcircle` - kitöltött kör.

Az 5 paraméteres és összetettebb hívásokhoz nem erőltetünk mesterséges targetenkénti regiszter-ABI-t. A könyvtár egységes, 16 bites **grafikai paraméterblokkot** használ:

| név | cím | jelentés |
|---|---:|---|
| `GFX_X0` | `0x00C0` | x / x0 / cx |
| `GFX_Y0` | `0x00C2` | y / y0 / cy |
| `GFX_X1` | `0x00C4` | x1 |
| `GFX_Y1` | `0x00C6` | y1 |
| `GFX_W` | `0x00C8` | szélesség |
| `GFX_H` | `0x00CA` | magasság |
| `GFX_R` | `0x00CC` | sugár |
| `GFX_COLOR` | `0x00CE` | színslot 0..3 |

A `0x00B0..0x00BE` tartomány azokon a targeteken belső virtuálisregiszter-scratch, ahol a magas szintű geometria memória-alapú loweringot használ; `0x00D0..0x00E7` a geometriai algoritmusok belső scratch területe; `0x00E8..0x00FA` az alap grafikai rutinok scratch/current-color területe. **A `graphics.asm` használatakor ezért a `0x00B0..0x00FA` RAM-tartomány grafikai könyvtári területnek számít.**

### Hívási séma

`line(x1,x2,y1,y2,color)` szemantikájú hívás:

```asm
; target-specifikus STORE műveletekkel:
; x1=10 -> GFX_X0, x2=309 -> GFX_X1, y1=10 -> GFX_Y0, y2=189 -> GFX_Y1, color=1
CALL line
```

`rect/fillrect(x,y,w,h,color)` a `GFX_X0`, `GFX_Y0`, `GFX_W`, `GFX_H`, `GFX_COLOR` mezőket használja.

`circle/fillcircle(cx,cy,r,color)` a `GFX_X0`, `GFX_Y0`, `GFX_R`, `GFX_COLOR` mezőket használja.

A konkrét target-szintaxisra példa: `svm_asm/examples/<arch>/graphics_library.asm`.

## Algoritmusok és határok

A `line` egész aritmetikájú DDA-vonalrajzolást használ. A 320x200-as képtartományon belül a köztes `i*minor_delta` szorzat legfeljebb `319*199`, ezért 16 biten elfér.

A `circle` és `fillcircle` egész négyzetösszeges oktáns-szkennelést használ; nincs lebegőpontos vagy négyzetgyök hardverigénye. A kézi assembly változatoknál a körnek teljesen a framebufferen belül kell lennie. A C könyvtár koordináta-ellenőrzést is végez.

## Framebuffer

A framebuffer 16000 bájt, egy bájt négy 2 bites képpontot tárol. A bal szélső pixel a 7..6, majd az 5..4, 3..2 és 1..0 biteket használja. A 0..3 pixelértékek a `0xFF0C..0xFF0F` MMIO palettaregisztereken keresztül választanak a 16 színű master palettából.

## Procedure-GC

Például ha a program csak `fillrect`-et hív, akkor a `fillrect -> hline -> putpixel` függőségi lánc marad meg. A `line`, `circle`, `fillcircle`, `vline` stb. nem növeli a binárist, ha nem elérhető.
